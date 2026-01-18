use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

use crate::config::FeedConfig;
use crate::error::FeedError;
use crate::models::PriceData;

use super::PriceFeed;

const JUPITER_API_URL: &str = "https://api.jup.ag/price/v2";

// Token mint addresses on Solana
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

pub struct JupiterFeed {
    client: Client,
    pair: String,
    base_mint: String,
    quote_mint: String,
    priority: u32,
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JupiterResponse {
    data: HashMap<String, JupiterPriceData>,
}

#[derive(Debug, Deserialize)]
struct JupiterPriceData {
    price: String,
}

impl JupiterFeed {
    pub fn new(config: &FeedConfig, client: Client) -> Self {
        let base_mint = token_to_mint(&config.base_token);
        let quote_mint = token_to_mint(&config.quote_token);
        let api_key = std::env::var("JUPITER_API_KEY").ok();

        Self {
            client,
            pair: config.pair(),
            base_mint,
            quote_mint,
            priority: config.priority,
            api_key,
        }
    }
}

fn token_to_mint(token: &str) -> String {
    match token.to_uppercase().as_str() {
        "SOL" => SOL_MINT.to_string(),
        "USDC" => USDC_MINT.to_string(),
        "USDT" => USDT_MINT.to_string(),
        other => other.to_string(), // Assume it's already a mint address
    }
}

#[async_trait]
impl PriceFeed for JupiterFeed {
    fn name(&self) -> &str {
        "jupiter"
    }

    fn pair(&self) -> &str {
        &self.pair
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    async fn fetch_price(&self) -> Result<PriceData, FeedError> {
        let url = format!(
            "{}?ids={}&vsToken={}",
            JUPITER_API_URL, self.base_mint, self.quote_mint
        );

        let mut request = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5));

        if let Some(ref api_key) = self.api_key {
            request = request.header("x-api-key", api_key);
        }

        let response: JupiterResponse = request.send().await?.json().await?;

        let price_data = response
            .data
            .get(&self.base_mint)
            .ok_or_else(|| FeedError::ParseError("Token not found in response".to_string()))?;

        let price = Decimal::from_str(&price_data.price)
            .map_err(|e| FeedError::ParseError(format!("Invalid price format: {}", e)))?;

        if price <= Decimal::ZERO {
            return Err(FeedError::InvalidData("Price must be positive".to_string()));
        }

        Ok(PriceData {
            pair: self.pair.clone(),
            price,
            source: self.name().to_string(),
            timestamp: Utc::now(),
        })
    }
}
