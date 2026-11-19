use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

use super::PriceFeed;
use crate::{config::FeedConfig, error::FeedError, models::PriceData};

const API: &str = "https://api.jup.ag/price/v3";
const SOL: &str = "So11111111111111111111111111111111111111112";
const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const ANT: &str = "ANTBTZQ1EUuTxUs3tfeampxdqTWPHmwyJmcWj4xoAyNe";

pub struct JupiterFeed {
    client: Client,
    pair: String,
    base_mint: String,
    quote_mint: String,
    priority: u32,
    api_key: Option<String>,
    rpc_url: String,
    api_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Price {
    usd_price: Decimal,
    block_id: u64,
}

impl JupiterFeed {
    pub fn new(config: &FeedConfig, client: Client) -> Self {
        Self {
            client,
            pair: config.pair(),
            base_mint: token_to_mint(&config.base_token),
            quote_mint: token_to_mint(&config.quote_token),
            priority: config.priority,
            api_key: std::env::var("JUPITER_API_KEY").ok(),
            rpc_url: std::env::var("PRICE_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into()),
            api_url: API.into(),
        }
    }

    async fn block_time(&self, slot: u64) -> Result<DateTime<Utc>, FeedError> {
        #[derive(Deserialize)]
        struct Response {
            result: Option<i64>,
        }
        let response: Response = self
            .client
            .post(&self.rpc_url)
            .timeout(Duration::from_secs(5))
            .json(&json!({"jsonrpc":"2.0", "id":1, "method":"getBlockTime", "params":[slot]}))
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)?;
        response
            .result
            .filter(|value| *value > 0)
            .and_then(|value| DateTime::from_timestamp(value, 0))
            .filter(|value| *value <= Utc::now() + chrono::Duration::seconds(5))
            .ok_or_else(|| {
                FeedError::InvalidData("missing or invalid price observation time".into())
            })
    }
}

fn http_error(error: reqwest::Error) -> FeedError {
    FeedError::HttpError(error.without_url())
}

fn token_to_mint(token: &str) -> String {
    match token.to_uppercase().as_str() {
        "SOL" => SOL.into(),
        "USDC" => USDC.into(),
        "ANT" => ANT.into(),
        "USDT" => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".into(),
        _ => token.into(),
    }
}

fn pair_price(
    response: &HashMap<String, Price>,
    base: &str,
    quote: &str,
) -> Result<(Decimal, u64, u64), FeedError> {
    let get = |mint| {
        response
            .get(mint)
            .filter(|p| p.usd_price > Decimal::ZERO && p.block_id > 0)
            .ok_or_else(|| {
                FeedError::InvalidData("missing or invalid token price/observation slot".into())
            })
    };
    let base = get(base)?;
    let quote = get(quote)?;
    let price = base
        .usd_price
        .checked_div(quote.usd_price)
        .filter(|p| *p > Decimal::ZERO)
        .ok_or_else(|| FeedError::InvalidData("invalid base/quote price ratio".into()))?;
    Ok((price, base.block_id, quote.block_id))
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
        let mut request = self
            .client
            .get(&self.api_url)
            .query(&[("ids", format!("{},{}", self.base_mint, self.quote_mint))])
            .timeout(Duration::from_secs(5));
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        let response: HashMap<String, Price> = request
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)?;
        let (price, base_slot, quote_slot) =
            pair_price(&response, &self.base_mint, &self.quote_mint)?;
        let (base_time, quote_time) =
            tokio::try_join!(self.block_time(base_slot), self.block_time(quote_slot))?;
        Ok(PriceData {
            pair: self.pair.clone(),
            price,
            source: self.name().into(),
            timestamp: base_time.min(quote_time),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use axum::{
        routing::{get, post},
        Json, Router,
    };

    #[test]
    fn ratio_uses_usdc_price_and_requires_observation_slots() {
        let response: HashMap<String, Price> = serde_json::from_value(json!({
            "ANT":{"usdPrice":99,"blockId":100}, "USDC":{"usdPrice":0.99,"blockId":99}
        }))
        .unwrap();
        assert_eq!(
            pair_price(&response, "ANT", "USDC").unwrap(),
            (Decimal::from(100), 100, 99)
        );
        assert!(pair_price(&response, "OTHER", "USDC").is_err());
        for value in [
            json!({"usdPrice":100}),
            json!({"usdPrice":100,"blockId":null}),
        ] {
            assert!(serde_json::from_value::<Price>(value).is_err());
        }
        for value in [0, -1] {
            let response = HashMap::from([
                (
                    "ANT".into(),
                    Price {
                        usd_price: Decimal::from(100),
                        block_id: 100,
                    },
                ),
                (
                    "USDC".into(),
                    Price {
                        usd_price: Decimal::from(value),
                        block_id: 100,
                    },
                ),
            ]);
            assert!(pair_price(&response, "ANT", "USDC").is_err());
        }
        assert_eq!(token_to_mint("ANT"), ANT);
        assert_eq!(token_to_mint(ANT), ANT);
    }

    #[tokio::test]
    async fn old_quotes_stay_stale_and_bad_rpc_times_are_rejected() {
        for timestamp in [
            Some(Utc::now().timestamp() - 120),
            None,
            Some(Utc::now().timestamp() + 60),
        ] {
            let router = Router::new()
                .route(
                    "/prices",
                    get(|| async {
                        Json(json!({
                            ANT:{"usdPrice":100,"blockId":100},USDC:{"usdPrice":1,"blockId":99}
                        }))
                    }),
                )
                .route(
                    "/rpc",
                    post(move || async move { Json(json!({"result":timestamp})) }),
                );
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });
            let feed = JupiterFeed {
                client: Client::new(),
                pair: "ANT/USDC".into(),
                base_mint: ANT.into(),
                quote_mint: USDC.into(),
                priority: 1,
                api_key: None,
                rpc_url: format!("http://{address}/rpc"),
                api_url: format!("http://{address}/prices"),
            };
            let state = AppState::new(30);
            for _ in 0..2 {
                let sample = feed.fetch_price().await;
                if timestamp.is_some_and(|t| t < Utc::now().timestamp()) {
                    let sample = sample.unwrap();
                    assert_eq!(sample.timestamp.timestamp(), timestamp.unwrap());
                    state.update_price(sample, 1).await;
                    assert!(state.get_price("ANT/USDC").await.is_none());
                } else {
                    assert!(sample.is_err());
                }
            }
            server.abort();
        }
    }
}
