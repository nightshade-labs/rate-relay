use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::PriceData;

/// Key for identifying a price entry: (pair, source)
type PriceKey = (String, String);

#[derive(Debug, Clone)]
struct PriceEntry {
    data: PriceData,
    priority: u32,
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<RwLock<StateInner>>,
    staleness_threshold_secs: i64,
}

struct StateInner {
    prices: HashMap<PriceKey, PriceEntry>,
}

impl AppState {
    pub fn new(staleness_threshold_secs: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateInner {
                prices: HashMap::new(),
            })),
            staleness_threshold_secs: staleness_threshold_secs as i64,
        }
    }

    /// Store a price update from a feed
    pub async fn update_price(&self, data: PriceData, priority: u32) {
        let key = (data.pair.clone(), data.source.clone());
        let entry = PriceEntry { data, priority };

        let mut state = self.inner.write().await;
        state.prices.insert(key, entry);
    }

    /// Get the best available price for a pair (lowest priority number that has fresh data)
    pub async fn get_price(&self, pair: &str) -> Option<(PriceData, bool)> {
        let state = self.inner.read().await;
        let now = Utc::now();
        let staleness_threshold = Duration::seconds(self.staleness_threshold_secs);

        // Collect all prices for this pair
        let mut candidates: Vec<&PriceEntry> = state
            .prices
            .iter()
            .filter(|((p, _), _)| p == pair)
            .map(|(_, entry)| entry)
            .filter(|entry| is_fresh(&entry.data.timestamp, &now, &staleness_threshold))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by priority (lower is better)
        candidates.sort_by_key(|e| e.priority);

        let best = candidates.first()?;
        let fallback_used = best.priority > 1;

        Some((best.data.clone(), fallback_used))
    }

    /// Check if we have any fresh data at all
    pub async fn has_fresh_data(&self) -> bool {
        let state = self.inner.read().await;
        let now = Utc::now();
        let staleness_threshold = Duration::seconds(self.staleness_threshold_secs);

        state
            .prices
            .values()
            .any(|entry| is_fresh(&entry.data.timestamp, &now, &staleness_threshold))
    }

}

fn is_fresh(timestamp: &DateTime<Utc>, now: &DateTime<Utc>, threshold: &Duration) -> bool {
    *now - *timestamp < *threshold
}
