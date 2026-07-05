use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub model: String,
    pub controllable: bool,
    pub retrievable: bool,
    pub is_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,

    // Capability flags
    pub supports_power: bool,
    pub supports_brightness: bool,
    pub supports_color: bool,
    pub supports_color_temp: bool,
    #[serde(default)]
    pub supports_scenes: bool,
}

impl From<govee_api2::Device> for Device {
    fn from(device: govee_api2::Device) -> Self {
        let is_group = device.is_group();
        let supports_power = device.supports_power();
        let supports_brightness = device.supports_brightness();
        let supports_color = device.supports_color();
        let supports_color_temp = device.supports_color_temp();
        let supports_scenes = device.supports_scenes();
        let controllable = supports_power;
        let retrievable = supports_brightness || supports_color;

        Self {
            id: device.device,
            name: device.device_name,
            model: device.sku,
            controllable,
            retrievable,
            is_group,
            device_type: device.device_type,
            supports_power,
            supports_brightness,
            supports_color,
            supports_color_temp,
            supports_scenes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub power: bool,
    pub brightness: Option<u8>,
    pub color: Option<RgbColor>,
    pub color_temp: Option<u16>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
