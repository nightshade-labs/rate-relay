mod binance;
mod jupiter;
mod mock;
mod pyth;

pub use binance::BinanceFeed;
pub use jupiter::JupiterFeed;
pub use mock::MockFeed;
pub use pyth::PythFeed;

use async_trait::async_trait;
use reqwest::Client;

use crate::config::FeedConfig;
use crate::error::FeedError;
use crate::models::PriceData;

#[async_trait]
pub trait PriceFeed: Send + Sync {
    /// Unique identifier for this feed (e.g., "jupiter", "pyth")
    fn name(&self) -> &str;

    /// The token pair this feed is configured for (e.g., "SOL/USDC")
    fn pair(&self) -> &str;

    /// Priority for fallback ordering (lower = higher priority)
    fn priority(&self) -> u32;

    /// Fetch the current price
    async fn fetch_price(&self) -> Result<PriceData, FeedError>;
}

/// Create a price feed from configuration
pub fn create_feed(
    config: &FeedConfig,
    http_client: Client,
) -> Result<Box<dyn PriceFeed>, FeedError> {
    match config.feed_type.as_str() {
        "jupiter" => Ok(Box::new(JupiterFeed::new(config, http_client))),
        "pyth" => Ok(Box::new(PythFeed::new(config, http_client))),
        "binance" => Ok(Box::new(BinanceFeed::new(config, http_client))),
        "mock" => Ok(Box::new(MockFeed::new(config, http_client))),
        other => Err(FeedError::NotImplemented(format!(
            "Unknown feed type: {}",
            other
        ))),
    }
}
