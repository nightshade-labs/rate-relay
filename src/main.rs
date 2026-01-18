mod api;
mod config;
mod error;
mod feeds;
mod metrics;
mod models;
mod scheduler;
mod state;

use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::create_router;
use crate::config::Config;
use crate::feeds::create_feed;
use crate::metrics::Metrics;
use crate::scheduler::FeedScheduler;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rate_relay=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Rate Relay service");

    // Load configuration
    let config = Config::load("config.toml")?;
    info!(port = config.server.port, "Configuration loaded");

    // Create shared state and metrics
    let app_state = AppState::new(config.server.staleness_threshold_secs);
    let metrics = Arc::new(Metrics::new());

    // Create HTTP client for all feeds
    let http_client = reqwest::Client::new();

    // Spawn feed schedulers for enabled feeds
    let enabled_feeds: Vec<_> = config.feeds.iter().filter(|f| f.enabled).collect();

    if enabled_feeds.is_empty() {
        warn!("No feeds enabled in configuration");
    }

    for feed_config in enabled_feeds {
        match create_feed(feed_config, http_client.clone()) {
            Ok(feed) => {
                let scheduler = FeedScheduler::new(
                    feed,
                    feed_config.interval_ms,
                    app_state.clone(),
                    metrics.clone(),
                );

                tokio::spawn(async move {
                    scheduler.run().await;
                });

                info!(
                    feed_type = %feed_config.feed_type,
                    pair = %feed_config.pair(),
                    interval_ms = %feed_config.interval_ms,
                    "Feed scheduler started"
                );
            }
            Err(e) => {
                warn!(
                    feed_type = %feed_config.feed_type,
                    error = %e,
                    "Failed to create feed, skipping"
                );
            }
        }
    }

    // Create and start HTTP server
    let router = create_router(app_state, metrics);

    let addr = format!("0.0.0.0:{}", config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(address = %addr, "HTTP server listening");

    axum::serve(listener, router).await?;

    Ok(())
}
