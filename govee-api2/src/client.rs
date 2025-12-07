use crate::error::{Error, Result};
use crate::types::*;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://openapi.api.govee.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Govee API client for the v2 router-based endpoints.
///
/// # Example
///
/// ```rust,no_run
/// use govee_api2::GoveeClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Simple construction
///     let client = GoveeClient::new("your-api-key");
///
///     // Or with builder for custom configuration
///     let client = GoveeClient::builder("your-api-key")
///         .timeout(std::time::Duration::from_secs(10))
///         .build()?;
///
///     let devices = client.get_devices().await?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct GoveeClient {
    api_key: String,
    client: reqwest::Client,
}

/// Builder for creating a [`GoveeClient`] with custom configuration.
#[derive(Clone)]
pub struct GoveeClientBuilder {
    api_key: String,
    timeout: Duration,
    user_agent: Option<String>,
}

impl GoveeClientBuilder {
    /// Create a new builder with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            timeout: DEFAULT_TIMEOUT,
            user_agent: None,
        }
    }

    /// Set the request timeout (default: 30 seconds).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a custom User-Agent header.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<GoveeClient> {
        let mut builder = reqwest::Client::builder().timeout(self.timeout);

        if let Some(ua) = self.user_agent {
            builder = builder.user_agent(ua);
        }

        let client = builder.build().map_err(Error::Request)?;

        Ok(GoveeClient {
            api_key: self.api_key,
            client,
        })
    }
}

impl GoveeClient {
    /// Create a new Govee API client with default settings.
    ///
    /// For custom configuration, use [`GoveeClient::builder`] instead.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a builder for custom client configuration.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use govee_api2::GoveeClient;
    /// use std::time::Duration;
    ///
    /// let client = GoveeClient::builder("your-api-key")
    ///     .timeout(Duration::from_secs(10))
    ///     .user_agent("my-app/1.0")
    ///     .build()
    ///     .expect("Failed to build client");
    /// ```
    pub fn builder(api_key: impl Into<String>) -> GoveeClientBuilder {
        GoveeClientBuilder::new(api_key)
    }

    /// List all devices and groups
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let url = format!("{}/router/api/v1/user/devices", API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Govee-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let api_response: ApiResponse<Vec<Device>> = response.json().await?;

        if api_response.code != 0 && api_response.code != 200 {
            return Err(Error::Api {
                code: api_response.code,
                message: api_response.message,
            });
        }

        Ok(api_response.data)
    }

    /// Get the current state of a device
    pub async fn get_device_state(&self, device_id: &str, sku: &str) -> Result<DeviceState> {
        let url = format!("{}/router/api/v1/device/state", API_BASE);

        let payload = json!({
            "requestId": generate_request_id(),
            "payload": {
                "sku": sku,
                "device": device_id
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Govee-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let api_response: DeviceStateResponse = response.json().await?;

        if api_response.code != 0 && api_response.code != 200 {
            return Err(Error::Api {
                code: api_response.code,
                message: api_response.msg,
            });
        }

        Ok(DeviceState::from_capabilities(
            api_response.payload.capabilities,
        ))
    }

    /// Turn a device on
    pub async fn turn_on(&self, device_id: &str, sku: &str) -> Result<()> {
        self.control_power(device_id, sku, PowerState::On).await
    }

    /// Turn a device off
    pub async fn turn_off(&self, device_id: &str, sku: &str) -> Result<()> {
        self.control_power(device_id, sku, PowerState::Off).await
    }

    /// Toggle device power
    async fn control_power(&self, device_id: &str, sku: &str, state: PowerState) -> Result<()> {
        let value = match state {
            PowerState::On => 1,
            PowerState::Off => 0,
        };

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.on_off",
            "powerSwitch",
            json!(value),
        )
        .await
    }

    /// Set device brightness (0-100)
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

    /// Set device color
    pub async fn set_color(&self, device_id: &str, sku: &str, color: Color) -> Result<()> {
        // Pack RGB into single integer: (r << 16) | (g << 8) | b
        let packed_rgb = ((color.r as i64) << 16) | ((color.g as i64) << 8) | (color.b as i64);

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.color_setting",
            "colorRgb",
            json!(packed_rgb),
        )
        .await
    }

    /// Set color temperature in Kelvin (2000-9000).
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

    /// Apply a dynamic scene (light effect) to the device.
    ///
    /// The `scene_id` should match one of the scene IDs from the device's
    /// `dynamicScene` capability parameters.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use govee_api2::GoveeClient;
    /// # async fn example() -> Result<(), govee_api2::Error> {
    /// let client = GoveeClient::new("your-api-key");
    /// // Apply "Sunrise" scene
    /// client.set_dynamic_scene("device-id", "H6159", 123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_dynamic_scene(&self, device_id: &str, sku: &str, scene_id: i32) -> Result<()> {
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.dynamic_scene",
            "lightScene",
            json!({ "id": scene_id }),
        )
        .await
    }

    /// Apply a DIY scene (user-created effect) to the device.
    ///
    /// The `diy_id` should match one of the DIY scene IDs from the device's
    /// `diyScene` capability parameters.
    pub async fn set_diy_scene(&self, device_id: &str, sku: &str, diy_id: i32) -> Result<()> {
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.dynamic_scene",
            "diyScene",
            json!({ "id": diy_id }),
        )
        .await
    }

    /// Set colors for individual segments on RGBIC (addressable LED) devices.
    ///
    /// Each segment can have its own color. The `segments` parameter is a list
    /// of (segment_index, color) pairs. Segment indices are 0-based.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use govee_api2::{GoveeClient, Color};
    /// # async fn example() -> Result<(), govee_api2::Error> {
    /// let client = GoveeClient::new("your-api-key");
    /// // Set first 3 segments to red, green, blue
    /// client.set_segment_colors("device-id", "H6167", &[
    ///     (0, Color::new(255, 0, 0)),
    ///     (1, Color::new(0, 255, 0)),
    ///     (2, Color::new(0, 0, 255)),
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_segment_colors(
        &self,
        device_id: &str,
        sku: &str,
        segments: &[(u8, Color)],
    ) -> Result<()> {
        let segment_data: Vec<serde_json::Value> = segments
            .iter()
            .map(|(idx, color)| {
                let packed_rgb =
                    ((color.r as i64) << 16) | ((color.g as i64) << 8) | (color.b as i64);
                json!({
                    "segment": idx,
                    "rgb": packed_rgb
                })
            })
            .collect();

        self.send_control(
            device_id,
            sku,
            "devices.capabilities.segment_color_setting",
            "segmentedColorRgb",
            json!(segment_data),
        )
        .await
    }

    /// Toggle a device feature on or off (e.g., gradient mode, nightlight).
    ///
    /// Common toggle instances:
    /// - `"gradientToggle"` - Enable/disable gradient mode
    /// - `"nightlightToggle"` - Enable/disable nightlight mode
    pub async fn set_toggle(
        &self,
        device_id: &str,
        sku: &str,
        instance: &str,
        enabled: bool,
    ) -> Result<()> {
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.toggle",
            instance,
            json!(if enabled { 1 } else { 0 }),
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
        let url = format!("{}/router/api/v1/device/control", API_BASE);

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

        let response = self
            .client
            .post(&url)
            .header("Govee-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let control_response: ControlResponse = response.json().await?;

        if control_response.code != 0 && control_response.code != 200 {
            return Err(Error::Api {
                code: control_response.code,
                message: control_response.msg,
            });
        }

        Ok(())
    }
}

fn generate_request_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("rust-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GoveeClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_builder_default() {
        let client = GoveeClient::builder("test-key").build().unwrap();
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_builder_with_timeout() {
        let client = GoveeClient::builder("test-key")
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_builder_with_user_agent() {
        let client = GoveeClient::builder("test-key")
            .user_agent("my-app/1.0")
            .build()
            .unwrap();
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_builder_full() {
        let client = GoveeClient::builder("test-key")
            .timeout(Duration::from_secs(10))
            .user_agent("test-agent")
            .build()
            .unwrap();
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_clone() {
        let client = GoveeClient::new("test-key");
        let cloned = client.clone();
        assert_eq!(cloned.api_key, "test-key");
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        assert!(id1.starts_with("rust-"));
        // Verify the ID contains a timestamp (digits after "rust-")
        let timestamp_str = &id1[5..];
        assert!(timestamp_str.chars().all(|c| c.is_ascii_digit()));
        assert!(!timestamp_str.is_empty());
    }

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.to_hex(), "#FF8040");
    }
}
