use prometheus::{self, Counter, CounterVec, Encoder, GaugeVec, Opts, Registry, TextEncoder};
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::FeedError;

pub struct Metrics {
    registry: Registry,
    fetch_total: CounterVec,
    fetch_errors: CounterVec,
    last_fetch_timestamp: GaugeVec,
    current_price: GaugeVec,
    http_requests: Counter,
    http_request_count: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let fetch_total = CounterVec::new(
            Opts::new("price_fetch_total", "Total number of price fetch attempts"),
            &["source", "pair", "result"],
        )
        .unwrap();

        let fetch_errors = CounterVec::new(
            Opts::new("price_fetch_errors_total", "Total number of failed price fetches"),
            &["source", "pair", "error_type"],
        )
        .unwrap();

        let last_fetch_timestamp = GaugeVec::new(
            Opts::new(
                "price_last_fetch_timestamp",
                "Unix timestamp of last successful fetch",
            ),
            &["source", "pair"],
        )
        .unwrap();

        let current_price = GaugeVec::new(
            Opts::new("price_current_value", "Current price value"),
            &["source", "pair"],
        )
        .unwrap();

        let http_requests = Counter::new("http_requests_total", "Total HTTP requests").unwrap();

        registry.register(Box::new(fetch_total.clone())).unwrap();
        registry.register(Box::new(fetch_errors.clone())).unwrap();
        registry
            .register(Box::new(last_fetch_timestamp.clone()))
            .unwrap();
        registry.register(Box::new(current_price.clone())).unwrap();
        registry.register(Box::new(http_requests.clone())).unwrap();

        Self {
            registry,
            fetch_total,
            fetch_errors,
            last_fetch_timestamp,
            current_price,
            http_requests,
            http_request_count: AtomicU64::new(0),
        }
    }

    pub fn record_fetch_success(&self, source: &str, pair: &str, price: &Decimal) {
        self.fetch_total
            .with_label_values(&[source, pair, "success"])
            .inc();

        self.last_fetch_timestamp
            .with_label_values(&[source, pair])
            .set(chrono::Utc::now().timestamp() as f64);

        if let Some(price_f64) = price.to_string().parse::<f64>().ok() {
            self.current_price
                .with_label_values(&[source, pair])
                .set(price_f64);
        }
    }

    pub fn record_fetch_error(&self, source: &str, pair: &str, error: &FeedError) {
        let error_type = match error {
            FeedError::HttpError(_) => "http",
            FeedError::ParseError(_) => "parse",
            FeedError::InvalidData(_) => "invalid_data",
            FeedError::NotImplemented(_) => "not_implemented",
        };

        self.fetch_total
            .with_label_values(&[source, pair, "error"])
            .inc();

        self.fetch_errors
            .with_label_values(&[source, pair, error_type])
            .inc();
    }

    pub fn record_http_request(&self, _endpoint: &str) {
        self.http_requests.inc();
        self.http_request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
