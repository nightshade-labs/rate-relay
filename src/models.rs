use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub pair: String,
    pub price: Decimal,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceResponse {
    pub pair: String,
    pub price: String,
    pub source: String,
    pub fallback_used: bool,
    pub timestamp: DateTime<Utc>,
}

impl PriceResponse {
    pub fn from_price_data(data: &PriceData, fallback_used: bool) -> Self {
        Self {
            pair: data.pair.clone(),
            price: data.price.to_string(),
            source: data.source.clone(),
            fallback_used,
            timestamp: data.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
