use async_trait::async_trait;
use reqwest::Client;

use crate::config::FeedConfig;
use crate::error::FeedError;
use crate::models::PriceData;

use super::PriceFeed;

/// Stub implementation for Pyth price feed
pub struct PythFeed {
    pair: String,
    priority: u32,
    #[allow(dead_code)]
    client: Client,
}

impl PythFeed {
    pub fn new(config: &FeedConfig, client: Client) -> Self {
        Self {
            pair: config.pair(),
            priority: config.priority,
            client,
        }
    }
}

#[async_trait]
impl PriceFeed for PythFeed {
    fn name(&self) -> &str {
        "pyth"
    }

    fn pair(&self) -> &str {
        &self.pair
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    async fn fetch_price(&self) -> Result<PriceData, FeedError> {
        Err(FeedError::NotImplemented(
            "Pyth feed not yet implemented".to_string(),
        ))
    }
}
