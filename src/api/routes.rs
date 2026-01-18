use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use std::sync::Arc;

use crate::metrics::Metrics;
use crate::models::{ErrorResponse, HealthResponse, PriceResponse};
use crate::state::AppState;

#[derive(Clone)]
pub struct ApiState {
    pub app_state: AppState,
    pub metrics: Arc<Metrics>,
}

pub fn create_router(app_state: AppState, metrics: Arc<Metrics>) -> Router {
    let api_state = ApiState { app_state, metrics };

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/price/:base/:quote", get(get_price))
        .route("/metrics", get(metrics_handler))
        .with_state(api_state)
}

async fn health(State(state): State<ApiState>) -> impl IntoResponse {
    let has_data = state.app_state.has_fresh_data().await;

    if has_data {
        (
            StatusCode::OK,
            Json(HealthResponse {
                status: "healthy".to_string(),
                timestamp: Utc::now(),
                reason: None,
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "unhealthy".to_string(),
                timestamp: Utc::now(),
                reason: Some("No fresh price data available".to_string()),
            }),
        )
    }
}

async fn get_price(
    State(state): State<ApiState>,
    Path((base, quote)): Path<(String, String)>,
) -> impl IntoResponse {
    let pair = format!("{}/{}", base.to_uppercase(), quote.to_uppercase());

    state.metrics.record_http_request(&format!("/api/v1/price/{}/{}", base, quote));

    match state.app_state.get_price(&pair).await {
        Some((price_data, fallback_used)) => {
            let response = PriceResponse::from_price_data(&price_data, fallback_used);
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
        }
        None => {
            let response = ErrorResponse {
                error: format!("No price data available for {}", pair),
            };
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(response).unwrap()),
            )
        }
    }
}

async fn metrics_handler(State(state): State<ApiState>) -> impl IntoResponse {
    state.metrics.encode()
}
