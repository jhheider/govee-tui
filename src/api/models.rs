use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub model: String,
    pub controllable: bool,
    pub retrievable: bool,
    pub online: bool,
    pub is_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
}

impl From<govee_api2::Device> for Device {
    fn from(device: govee_api2::Device) -> Self {
        let is_group = device.is_group();
        let controllable = device.supports_power();
        let retrievable = device.supports_brightness() || device.supports_color();
        
        Self {
            id: device.device,
            name: device.device_name,
            model: device.sku,
            controllable,
            retrievable,
            online: true,
            is_group,
            device_type: device.device_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub online: bool,
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

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}
