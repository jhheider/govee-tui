use crate::error::{Error, Result};
use crate::types::*;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://openapi.api.govee.com";

/// Govee API client for the v2 router-based endpoints
#[derive(Clone)]
pub struct GoveeClient {
    api_key: String,
    client: reqwest::Client,
}

impl GoveeClient {
    /// Create a new Govee API client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
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
            return Err(Error::InvalidResponse(format!(
                "HTTP {}: {}",
                status, body
            )));
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
            return Err(Error::InvalidResponse(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let api_response: ApiResponse<DeviceStateResponse> = response.json().await?;

        if api_response.code != 0 && api_response.code != 200 {
            return Err(Error::Api {
                code: api_response.code,
                message: api_response.message,
            });
        }

        Ok(DeviceState::from_capabilities(api_response.data.capabilities))
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

        self.send_control(device_id, sku, "devices.capabilities.on_off", "powerSwitch", json!(value))
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
        self.send_control(
            device_id,
            sku,
            "devices.capabilities.color_setting",
            "colorRgb",
            json!({
                "r": color.r,
                "g": color.g,
                "b": color.b
            }),
        )
        .await
    }

    /// Set color temperature in Kelvin (2000-9000)
    pub async fn set_color_temperature(&self, device_id: &str, sku: &str, kelvin: i32) -> Result<()> {
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
            return Err(Error::InvalidResponse(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;

        if api_response.code != 0 && api_response.code != 200 {
            return Err(Error::Api {
                code: api_response.code,
                message: api_response.message,
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
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert!(id1.starts_with("rust-"));
        assert_ne!(id1, id2);
    }
}
