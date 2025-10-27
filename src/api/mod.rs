use anyhow::{Context, Result};
use govee_api::{structs::govee::PayloadBody, GoveeClient};
use tracing::{debug, info};

pub mod commands;
pub mod models;

pub use commands::Command;
pub use models::Device;

pub struct Client {
    inner: GoveeClient,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self> {
        let inner = GoveeClient::new(api_key);
        info!("Govee API client initialized");
        Ok(Self { inner })
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching device list");
        let response = self.inner.get_devices().await.context("Failed to fetch devices")?;

        let devices: Vec<Device> = response
            .data
            .context("No device data in response")?
            .devices
            .into_iter()
            .map(Device::from)
            .collect();

        info!("Fetched {} devices", devices.len());
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

        self.inner.control_device(payload).await.context("Failed to control device")?;

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
