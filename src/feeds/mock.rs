use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::FeedConfig;
use crate::error::FeedError;
use crate::models::PriceData;

use super::PriceFeed;

/// Mock price feed for testing - generates simulated price data
pub struct MockFeed {
    pair: String,
    priority: u32,
    base_price: Decimal,
    counter: AtomicU64,
}

impl MockFeed {
    pub fn new(config: &FeedConfig, _client: Client) -> Self {
        // Default base price for different pairs
        let base_price = match (config.base_token.as_str(), config.quote_token.as_str()) {
            ("SOL", "USDC") | ("SOL", "USDT") => Decimal::from_str("180.0").unwrap(),
            ("BTC", "USDC") | ("BTC", "USDT") => Decimal::from_str("95000.0").unwrap(),
            ("ETH", "USDC") | ("ETH", "USDT") => Decimal::from_str("3200.0").unwrap(),
            _ => Decimal::from_str("100.0").unwrap(),
        };

        Self {
            pair: config.pair(),
            priority: config.priority,
            base_price,
            counter: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl PriceFeed for MockFeed {
    fn name(&self) -> &str {
        "mock"
    }

    fn pair(&self) -> &str {
        &self.pair
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    async fn fetch_price(&self) -> Result<PriceData, FeedError> {
        // Generate a price that fluctuates slightly around the base price
        let count = self.counter.fetch_add(1, Ordering::Relaxed);

        // Simple oscillation: +/- 0.5% based on counter
        let oscillation = ((count % 100) as i64 - 50) as f64 / 10000.0;
        let multiplier = Decimal::from_str(&format!("{:.6}", 1.0 + oscillation)).unwrap();
        let price = self.base_price * multiplier;

        Ok(PriceData {
            pair: self.pair.clone(),
            price,
            source: self.name().to_string(),
            timestamp: Utc::now(),
        })
    }
}
