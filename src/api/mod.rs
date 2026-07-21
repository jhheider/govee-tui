use color_eyre::eyre::{Result, WrapErr};
use govee_api2::{ClientConfig, GoveeClient};
use std::time::Duration;
use tracing::{debug, info, warn};

pub mod commands;
pub mod models;

pub use commands::Command;
pub use govee_api2::Scene;
pub use models::Device;
pub use models::DeviceState;

#[derive(Clone)]
pub struct Client {
    inner: GoveeClient,
}

impl Client {
    pub fn new(config: &crate::config::ApiConfig) -> Result<Self> {
        let inner = GoveeClient::with_config(
            &config.key,
            ClientConfig {
                timeout: Duration::from_millis(config.timeout_ms),
                retry_attempts: config.retry_attempts,
                ..ClientConfig::default()
            },
        );
        info!("Govee API client initialized (govee-api2)");
        Ok(Self { inner })
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching device list from Govee API v2");

        let devices = self
            .inner
            .get_devices()
            .await
            .wrap_err("Failed to fetch devices")?;

        info!("Successfully fetched {} devices from API", devices.len());

        let converted: Vec<Device> = devices.into_iter().map(Device::from).collect();

        for (i, device) in converted.iter().enumerate() {
            let type_str = if device.is_group { "GROUP" } else { "DEVICE" };
            debug!(
                "  {} {}: {} ({})",
                type_str,
                i + 1,
                device.name,
                device.model
            );
        }

        Ok(converted)
    }

    pub async fn control_device(
        &self,
        device_id: &str,
        model: &str,
        command: Command,
    ) -> Result<()> {
        debug!("Sending command {:?} to device {}", command, device_id);

        match command {
            Command::TurnOn => {
                self.inner.turn_on(device_id, model).await?;
            }
            Command::TurnOff => {
                self.inner.turn_off(device_id, model).await?;
            }
            Command::Brightness(value) => {
                self.inner.set_brightness(device_id, model, value).await?;
            }
            Command::Color(r, g, b) => {
                let color = govee_api2::Color::new(r, g, b);
                self.inner.set_color(device_id, model, color).await?;
            }
            Command::ColorTemp(kelvin) => {
                self.inner
                    .set_color_temperature(device_id, model, kelvin as i32)
                    .await?;
            }
            Command::Scene(scene) => {
                self.inner.set_scene(device_id, model, &scene).await?;
            }
        }

        info!("Device {} controlled successfully", device_id);
        Ok(())
    }

    /// Dynamic light scenes plus the user's DIY scenes. A missing/failed
    /// DIY list doesn't fail the call; not every device or account has one.
    pub async fn get_scenes(&self, device_id: &str, model: &str) -> Result<Vec<Scene>> {
        debug!("Fetching scenes for device {}", device_id);

        let mut scenes = self
            .inner
            .get_dynamic_scenes(device_id, model)
            .await
            .wrap_err("Failed to fetch scenes")?;

        match self.inner.get_diy_scenes(device_id, model).await {
            Ok(diy) => scenes.extend(diy),
            Err(e) => warn!("Failed to fetch DIY scenes for {device_id}: {e}"),
        }

        info!("Fetched {} scenes for device {}", scenes.len(), device_id);
        Ok(scenes)
    }

    pub async fn get_device_state(
        &self,
        device_id: &str,
        model: &str,
    ) -> Result<models::DeviceState> {
        debug!("Fetching state for device {}", device_id);

        let state = self
            .inner
            .get_device_state(device_id, model)
            .await
            .wrap_err("Failed to get device state")?;

        // Convert govee_api2::DeviceState to our DeviceState
        Ok(models::DeviceState {
            power: state.power,
            brightness: state.brightness.map(|b| b as u8),
            color: state.color.map(|c| models::RgbColor::new(c.r, c.g, c.b)),
            color_temp: state.color_temperature_kelvin.map(|k| k as u16),
        })
    }
}
