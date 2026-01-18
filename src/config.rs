use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    #[serde(default)]
    pub feeds: Vec<FeedConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_staleness_threshold_secs")]
    pub staleness_threshold_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeedConfig {
    #[serde(rename = "type")]
    pub feed_type: String,
    pub base_token: String,
    pub quote_token: String,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_port() -> u16 {
    8080
}

fn default_staleness_threshold_secs() -> u64 {
    30
}

fn default_interval_ms() -> u64 {
    1500
}

fn default_priority() -> u32 {
    100
}

fn default_enabled() -> bool {
    true
}

impl FeedConfig {
    pub fn pair(&self) -> String {
        format!("{}/{}", self.base_token, self.quote_token)
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            staleness_threshold_secs: default_staleness_threshold_secs(),
        }
    }
}
