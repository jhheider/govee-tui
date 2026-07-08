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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_request_display() {
        let err = Error::InvalidApiKey;
        assert_eq!(
            err.to_string(),
            "invalid API key: the Govee API rejected the Govee-API-Key header"
        );
    }

    #[test]
    fn error_rate_limited_no_retry() {
        let err = Error::RateLimited { retry_after_secs: None };
        assert_eq!(err.to_string(), "rate limited by the Govee API");
    }

    #[test]
    fn error_rate_limited_with_retry() {
        let err = Error::RateLimited {
            retry_after_secs: Some(30),
        };
        assert_eq!(
            err.to_string(),
            "rate limited by the Govee API (retry after 30s)"
        );
    }

    #[test]
    fn error_server_display() {
        let err = Error::Server { status: 503 };
        assert_eq!(err.to_string(), "Govee API server error (HTTP 503)");
    }

    #[test]
    fn error_api_display() {
        let err = Error::Api {
            code: 400,
            message: "bad request".into(),
        };
        assert_eq!(err.to_string(), "API error (code 400): bad request");
    }

    #[test]
    fn error_invalid_response_display() {
        let err = Error::InvalidResponse("unexpected EOF".into());
        assert_eq!(
            err.to_string(),
            "Invalid response format: unexpected EOF"
        );
    }

    #[test]
    fn error_device_not_found() {
        let err = Error::DeviceNotFound("AA:BB:CC".into());
        assert_eq!(err.to_string(), "Device not found: AA:BB:CC");
    }

    #[test]
    fn error_type_impls_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Error>();
        assert_sync::<Error>();
    }
}
