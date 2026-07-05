use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// A transport-level failure (connection, TLS, timeout, ...).
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// The Govee API returned a non-success application code.
    #[error("API error (code {code}): {message}")]
    Api { code: i32, message: String },

    /// The API key was rejected (HTTP 401/403).
    #[error("invalid API key: the Govee API rejected the Govee-API-Key header")]
    InvalidApiKey,

    /// The account hit Govee's request limit (HTTP 429; 10000 requests/account/day).
    #[error("rate limited by the Govee API{}", match .retry_after_secs {
        Some(secs) => format!(" (retry after {secs}s)"),
        None => String::new(),
    })]
    RateLimited { retry_after_secs: Option<u64> },

    /// The Govee API returned a server error (HTTP 5xx), even after retries.
    #[error("Govee API server error (HTTP {status})")]
    Server { status: u16 },

    #[error("Failed to parse JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}
