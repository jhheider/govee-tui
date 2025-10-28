use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error (code {code}): {message}")]
    Api { code: i32, message: String },

    #[error("Failed to parse JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}
