use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeedError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Invalid price data: {0}")]
    InvalidData(String),

    #[error("Feed not implemented: {0}")]
    NotImplemented(String),
}
