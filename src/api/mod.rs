use anyhow::{Context, Result};
use govee_api::{structs::govee::PayloadBody, GoveeClient};
use std::sync::Arc;
use tracing::{debug, info};

pub mod commands;
pub mod models;
pub mod new_client;

pub use commands::Command;
pub use models::Device;
pub use models::DeviceState;

#[derive(Clone)]
pub struct Client {
    inner: Arc<GoveeClient>,
    new_client: new_client::NewApiClient,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self> {
        let inner = Arc::new(GoveeClient::new(api_key));
        let new_client = new_client::NewApiClient::new(api_key.to_string());
        info!("Govee API client initialized (using new router API for device list)");
        Ok(Self { inner, new_client })
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching device list from NEW Govee API (/router/api/v1/user/devices)");

        let response = self
            .new_client
            .get_devices()
            .await
            .context("Failed to fetch devices from new API")?;

        debug!("Raw API response code: {}, message: {}", response.code, response.message);
        debug!("Received {} devices from new API", response.data.len());

        let devices: Vec<Device> = response
            .data
            .into_iter()
            .map(|new_dev| {
                let is_group = new_dev.sku == "SameModeGroup";
                Device {
                    id: new_dev.device,
                    name: new_dev.device_name,
                    model: new_dev.sku.clone(),
                    controllable: new_dev.capabilities.iter().any(|c| {
                        c.capability_type == "devices.capabilities.on_off"
                    }),
                    retrievable: new_dev.capabilities.iter().any(|c| {
                        c.capability_type == "devices.capabilities.range"
                            || c.capability_type == "devices.capabilities.color_setting"
                    }),
                    online: true, // New API doesn't provide online status in device list
                    is_group,
                    device_type: new_dev.device_type,
                }
            })
            .collect();

        info!("Successfully parsed {} devices from new API", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let type_str = if device.is_group { "GROUP" } else { "DEVICE" };
            debug!("  {} {}: {} ({}) - controllable: {}",
                type_str, i + 1, device.name, device.model, device.controllable);
        }

        Ok(devices)
    }

    pub async fn control_device(
        &self,
        device_id: &str,
        model: &str,
        command: Command,
    ) -> Result<()> {
        debug!("Sending command {:?} to device {}", command, device_id);

        let payload = PayloadBody {
            device: device_id.to_string(),
            model: model.to_string(),
            cmd: command.to_govee_command(),
        };

        self.inner
            .control_device(payload)
            .await
            .context("Failed to control device")?;

        info!("Device {} controlled successfully", device_id);
        Ok(())
    }

    pub async fn get_device_state(
        &self,
        device_id: &str,
        model: &str,
    ) -> Result<models::DeviceState> {
        debug!("Fetching state for device {}", device_id);

        let response = self
            .inner
            .get_device_state(device_id, model)
            .await
            .context("Failed to get device state")?;

        let data = response.data.context("No state data in response")?;
        let state = models::DeviceState::from(data);
        Ok(state)
    }
}
