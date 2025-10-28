use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GOVEE_API_BASE: &str = "https://openapi.api.govee.com";

/// New API client using the router-based endpoints
#[derive(Clone)]
pub struct NewApiClient {
    api_key: String,
    client: reqwest::Client,
}

impl NewApiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_devices(&self) -> Result<NewDeviceListResponse> {
        let url = format!("{}/router/api/v1/user/devices", GOVEE_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Govee-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed ({}): {}", status, body);
        }

        // Get raw text first to debug
        let text = response.text().await.context("Failed to read response body")?;

        // Try to parse the response
        let data: NewDeviceListResponse = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse JSON response. First 200 chars: {}",
                &text.chars().take(200).collect::<String>()))?;
        Ok(data)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewDeviceListResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub message: String,
    pub data: Vec<NewDevice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewDevice {
    pub sku: String,
    pub device: String,
    pub device_name: String,
    #[serde(rename = "type")]
    pub device_type: Option<String>, // Missing for device groups!
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    #[serde(rename = "type")]
    pub capability_type: String,
    pub instance: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
}
