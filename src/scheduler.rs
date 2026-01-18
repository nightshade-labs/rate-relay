use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

use crate::feeds::PriceFeed;
use crate::metrics::Metrics;
use crate::state::AppState;

pub struct FeedScheduler {
    feed: Box<dyn PriceFeed>,
    interval_ms: u64,
    state: AppState,
    metrics: Arc<Metrics>,
}

impl FeedScheduler {
    pub fn new(
        feed: Box<dyn PriceFeed>,
        interval_ms: u64,
        state: AppState,
        metrics: Arc<Metrics>,
    ) -> Self {
        Self {
            feed,
            interval_ms,
            state,
            metrics,
        }
    }

    pub async fn run(self) {
        let feed_name = self.feed.name().to_string();
        let pair = self.feed.pair().to_string();
        let priority = self.feed.priority();

        info!(
            feed = %feed_name,
            pair = %pair,
            interval_ms = %self.interval_ms,
            "Starting price feed scheduler"
        );

        let mut ticker = interval(Duration::from_millis(self.interval_ms));

        loop {
            ticker.tick().await;

            match self.feed.fetch_price().await {
                Ok(price_data) => {
                    info!(
                        feed = %feed_name,
                        pair = %pair,
                        price = %price_data.price,
                        "Fetched price"
                    );

                    self.metrics
                        .record_fetch_success(&feed_name, &pair, &price_data.price);
                    self.state.update_price(price_data, priority).await;
                }
                Err(e) => {
                    error!(
                        feed = %feed_name,
                        pair = %pair,
                        error = %e,
                        "Failed to fetch price"
                    );

                    self.metrics.record_fetch_error(&feed_name, &pair, &e);
                }
            }
        }
    }
}
