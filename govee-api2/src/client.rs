use crate::error::{Error, Result};
use crate::types::*;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://openapi.api.govee.com";

/// Base delay for exponential retry backoff (doubles per attempt).
const RETRY_BASE_DELAY: Duration = Duration::from_millis(100);

/// Configuration for a [`GoveeClient`].
///
/// Controls timeout, retry behaviour, and the API base URL (useful for
/// testing against a mock server).
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Per-request timeout (default: 10 seconds)
    pub timeout: Duration,

    /// Number of retries after a failed attempt, for transport errors and
    /// HTTP 5xx responses (default: 3). Rate-limit (429) responses are
    /// never retried.
    pub retry_attempts: u32,

    /// API base URL. Override for testing against a mock server
    /// (default: `https://openapi.api.govee.com`).
    pub base_url: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            retry_attempts: 3,
            base_url: API_BASE.to_string(),
        }
    }
}

/// Govee API client for the v2 router-based endpoints
#[derive(Clone)]
pub struct GoveeClient {
    api_key: String,
    client: reqwest::Client,
    config: ClientConfig,
}

impl GoveeClient {
    /// Create a new Govee API client with default configuration
    /// (10 second timeout, 3 retries).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use govee_api2::GoveeClient;
    /// let client = GoveeClient::new("your-api-key");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, ClientConfig::default())
    }

    /// Create a new Govee API client with a custom configuration.
    ///
    /// Use this when you need to customise the timeout, retry policy, or
    /// point the client at a mock server for testing.
    pub fn with_config(api_key: impl Into<String>, config: ClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("failed to build reqwest client");

        Self {
            api_key: api_key.into(),
            client,
            config,
        }
    }

    /// List all devices and groups associated with the API key.
    ///
    /// Calls `GET /router/api/v1/user/devices`. Each device includes its
    /// capabilities, which can be queried with the convenience methods on
    /// [`Device`] (e.g. [`Device::supports_color`]).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidApiKey`] if the key is rejected, or
    /// [`Error::RateLimited`] if the account's daily quota is exhausted.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let response = self.request("/router/api/v1/user/devices", None).await?;
        let api_response: ApiResponse<Vec<Device>> = Self::parse_json(response).await?;

        Self::check_api_code(api_response.code, &api_response.message)?;

        Ok(api_response.data)
    }

    /// Get the current state of a device.
    ///
    /// Calls `POST /router/api/v1/device/state`. Returns power, brightness,
    /// colour, colour temperature, online status, active scenes, and whether
    /// the device supports segmented control.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeviceNotFound`] if the device identifier is unknown
    /// to the API key's account.
    pub async fn get_device_state(&self, device_id: &str, sku: &str) -> Result<DeviceState> {
        let payload = json!({
            "requestId": generate_request_id(),
            "payload": {
                "sku": sku,
                "device": device_id
            }
        });

        let response = self
            .request("/router/api/v1/device/state", Some(payload))
            .await?;
        let api_response: DeviceStateResponse = Self::parse_json(response).await?;

        Self::check_api_code(api_response.code, &api_response.msg)?;

        Ok(DeviceState::from_capabilities(
            api_response.payload.capabilities,
        ))
    }

    /// List the dynamic light scenes available for a device
    /// (`POST /router/api/v1/device/scenes`).
    ///
    /// Returns scenes built into the device (e.g. "Sunrise", "Night").
    /// See also [`Self::get_diy_scenes`] for user-created scenes.
    pub async fn get_dynamic_scenes(&self, device_id: &str, sku: &str) -> Result<Vec<Scene>> {
        self.get_scene_list("/router/api/v1/device/scenes", device_id, sku)
            .await
    }

    /// List the user-created DIY scenes available for a device
    /// (`POST /router/api/v1/device/diy-scenes`).
    ///
    /// DIY scenes are custom scenes the user created in the Govee Home app.
    /// These may not be available on every device or account.
    pub async fn get_diy_scenes(&self, device_id: &str, sku: &str) -> Result<Vec<Scene>> {
        self.get_scene_list("/router/api/v1/device/diy-scenes", device_id, sku)
            .await
    }

    async fn get_scene_list(&self, path: &str, device_id: &str, sku: &str) -> Result<Vec<Scene>> {
        let payload = json!({
            "requestId": generate_request_id(),
            "payload": {
                "sku": sku,
                "device": device_id
            }
        });

        let response = self.request(path, Some(payload)).await?;
        let api_response: ScenesResponse = Self::parse_json(response).await?;

        Self::check_api_code(api_response.code, &api_response.msg)?;

        Ok(Scene::from_capabilities(&api_response.payload.capabilities))
    }

    /// Activate a scene previously returned by [`Self::get_dynamic_scenes`]
    /// or [`Self::get_diy_scenes`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use govee_api2::GoveeClient;
    /// let client = GoveeClient::new("key");
    /// let scenes = client.get_dynamic_scenes("device-id", "H6072").await.unwrap();
    /// if let Some(scene) = scenes.first() {
    ///     client.set_scene("device-id", "H6072", scene).await.unwrap();
    /// }
    /// ```
    pub async fn set_scene(&self, device_id: &str, sku: &str, scene: &Scene) -> Result<()> {
        self.send_control(
            device_id,
            sku,
            &scene.capability_type,
            &scene.instance,
            scene.control_value(),
        )
        .await
    }

    /// Turn a device on.
    ///
    /// Sends an `on_off` command with value `1`.
    pub async fn turn_on(&self, device_id: &str, sku: &str) -> Result<()> {
        self.control_power(device_id, sku, PowerState::On).await
    }

    /// Turn a device off.
    ///
    /// Sends an `on_off` command with value `0`.
    pub async fn turn_off(&self, device_id: &str, sku: &str) -> Result<()> {
        self.control_power(device_id, sku, PowerState::Off).await
    }

    /// Toggle device power
    async fn control_power(&self, device_id: &str, sku: &str, state: PowerState) -> Result<()> {
        let value = if state.is_on() { 1 } else { 0 };

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.on_off",
            "powerSwitch",
            json!(value),
        )
        .await
    }

    /// Set device brightness (0-100).
    ///
    /// Values above 100 are silently clamped.
    pub async fn set_brightness(&self, device_id: &str, sku: &str, brightness: u8) -> Result<()> {
        let brightness = brightness.min(100);
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.range",
            "brightness",
            json!(brightness),
        )
        .await
    }

    /// Set device colour.
    ///
    /// Sends an RGB value as a packed integer `(r << 16) | (g << 8) | b`.
    pub async fn set_color(&self, device_id: &str, sku: &str, color: Color) -> Result<()> {
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.color_setting",
            "colorRgb",
            json!(color.to_packed()),
        )
        .await
    }

    /// Set colour temperature in Kelvin (2000-9000).
    ///
    /// Values outside the range are silently clamped.
    pub async fn set_color_temperature(
        &self,
        device_id: &str,
        sku: &str,
        kelvin: i32,
    ) -> Result<()> {
        let kelvin = kelvin.clamp(2000, 9000);
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.color_setting",
            "colorTemperatureK",
            json!(kelvin),
        )
        .await
    }

    /// Set the colour of specific segments of a segmented light.
    ///
    /// `segments` are zero-based segment indices as advertised by the
    /// device's `segmentedColorRgb` capability.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use govee_api2::GoveeClient;
    /// let client = GoveeClient::new("key");
    /// // Set segments 0 and 1 to red
    /// client.set_segment_color("device-id", "H6072", &[0, 1], 255, 0, 0).await.unwrap();
    /// ```
    pub async fn set_segment_color(
        &self,
        device_id: &str,
        sku: &str,
        segments: &[u8],
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<()> {
        let value = json!({
            "segment": segments,
            "rgb": Color::new(r, g, b).to_packed(),
        });

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.segment_color_setting",
            "segmentedColorRgb",
            value,
        )
        .await
    }

    /// Set the brightness of specific segments of a segmented light.
    ///
    /// Each segment receives the same brightness value (0-100).
    pub async fn set_segment_brightness(
        &self,
        device_id: &str,
        sku: &str,
        segments: &[u8],
        brightness: u8,
    ) -> Result<()> {
        let value = json!({
            "segment": segments,
            "brightness": brightness.min(100),
        });

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.segment_color_setting",
            "segmentedBrightness",
            value,
        )
        .await
    }

    /// Send a control command to a device
    async fn send_control(
        &self,
        device_id: &str,
        sku: &str,
        capability_type: &str,
        instance: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        let payload = ControlRequest {
            request_id: generate_request_id(),
            payload: ControlPayload {
                sku: sku.to_string(),
                device: device_id.to_string(),
                capability: CapabilityCommand {
                    capability_type: capability_type.to_string(),
                    instance: instance.to_string(),
                    value,
                },
            },
        };
        let payload = serde_json::to_value(&payload)?;

        let response = self
            .request("/router/api/v1/device/control", Some(payload))
            .await?;
        let control_response: ControlResponse = Self::parse_json(response).await?;

        Self::check_api_code(control_response.code, &control_response.msg)?;

        Ok(())
    }

    /// Perform a request with retry/backoff. `body: None` sends a GET,
    /// `Some(json)` a POST.
    ///
    /// Transport errors and HTTP 5xx responses are retried up to
    /// `retry_attempts` times with exponential backoff; 429 responses are
    /// surfaced immediately as [`Error::RateLimited`].
    async fn request(
        &self,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut last_error: Option<Error> = None;

        for attempt in 0..=self.config.retry_attempts {
            if attempt > 0 {
                tokio::time::sleep(RETRY_BASE_DELAY * 2u32.pow(attempt - 1)).await;
            }

            let request = match &body {
                Some(body) => self.client.post(&url).json(body),
                None => self.client.get(&url),
            }
            .header("Govee-API-Key", &self.api_key)
            .header("Content-Type", "application/json");

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        return Ok(response);
                    }

                    if status.is_server_error() {
                        // Retryable
                        last_error = Some(Error::Server {
                            status: status.as_u16(),
                        });
                        continue;
                    }

                    // Non-retryable client errors
                    return Err(match status.as_u16() {
                        401 | 403 => Error::InvalidApiKey,
                        429 => Error::RateLimited {
                            retry_after_secs: parse_retry_after(response.headers()),
                        },
                        _ => {
                            let body = response.text().await.unwrap_or_default();
                            Error::InvalidResponse(format!("HTTP {}: {}", status, body))
                        }
                    });
                }
                Err(err) => {
                    // Transport error (connection failure, timeout, ...):
                    // retryable
                    last_error = Some(Error::Request(err));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::InvalidResponse("request failed without a recorded error".to_string())
        }))
    }

    async fn parse_json<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
        Ok(response.json().await?)
    }

    fn check_api_code(code: i32, message: &str) -> Result<()> {
        if code != 0 && code != 200 {
            return Err(Error::Api {
                code,
                message: message.to_string(),
            });
        }
        Ok(())
    }
}

/// Extract a retry delay in seconds from rate-limit response headers.
///
/// Govee does not document specific headers for the platform API, so this
/// checks the standard `Retry-After` plus the `X-RateLimit-Reset` /
/// `API-RateLimit-Reset` variants used by the older Govee developer API
/// (which carry a UTC epoch timestamp).
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    if let Some(secs) = header_u64(headers, "Retry-After") {
        return Some(secs);
    }

    for name in ["X-RateLimit-Reset", "API-RateLimit-Reset"] {
        if let Some(value) = header_u64(headers, name) {
            // Values this large are epoch timestamps; convert to a delta.
            return Some(if value > 1_000_000_000 {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                value.saturating_sub(now)
            } else {
                value
            });
        }
    }

    None
}

fn header_u64(headers: &reqwest::header::HeaderMap, name: &str) -> Option<u64> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

fn generate_request_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("rust-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GoveeClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.config.timeout, Duration::from_secs(10));
        assert_eq!(client.config.retry_attempts, 3);
        assert_eq!(client.config.base_url, API_BASE);
    }

    #[test]
    fn test_client_with_config() {
        let client = GoveeClient::with_config(
            "test-key",
            ClientConfig {
                timeout: Duration::from_millis(250),
                retry_attempts: 0,
                base_url: "http://localhost:1234".to_string(),
            },
        );
        assert_eq!(client.config.timeout, Duration::from_millis(250));
        assert_eq!(client.config.retry_attempts, 0);
        assert_eq!(client.config.base_url, "http://localhost:1234");
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert!(id1.starts_with("rust-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_parse_retry_after_seconds() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Retry-After", "30".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), Some(30));
    }

    #[test]
    fn test_parse_retry_after_epoch_reset() {
        let mut headers = reqwest::header::HeaderMap::new();
        let reset = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        headers.insert("X-RateLimit-Reset", reset.to_string().parse().unwrap());
        let secs = parse_retry_after(&headers).unwrap();
        assert!((59..=61).contains(&secs), "expected ~60s, got {secs}");
    }
}
