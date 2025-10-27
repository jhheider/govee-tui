use govee_api::structs::govee::{GoveeDataDeviceStatus, GoveeDevice, GoveeDeviceProperty};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub model: String,
    pub controllable: bool,
    pub retrievable: bool,
    pub online: bool,
}

impl From<GoveeDevice> for Device {
    fn from(device: GoveeDevice) -> Self {
        Self {
            id: device.device,
            name: device.deviceName,
            model: device.model,
            controllable: device.controllable,
            retrievable: device.retrievable,
            online: true, // Default to online
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

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl From<GoveeDataDeviceStatus> for DeviceState {
    fn from(data: GoveeDataDeviceStatus) -> Self {
        let mut state = DeviceState {
            online: false,
            power: false,
            brightness: None,
            color: None,
            color_temp: None,
        };

        for prop in data.properties {
            match prop {
                GoveeDeviceProperty::Online(v) => state.online = v,
                GoveeDeviceProperty::PowerState(v) => state.power = v == "on",
                GoveeDeviceProperty::Brightness(v) => state.brightness = Some(v as u8),
                GoveeDeviceProperty::Color(_c) => {
                    // Note: Color fields in govee_api are private
                    state.color = Some(RgbColor::new(255, 255, 255));
                }
                GoveeDeviceProperty::ColorTem(v) | GoveeDeviceProperty::ColorTemInKelvin(v) => {
                    state.color_temp = Some(v as u16);
                }
            }
        }

        state
    }
}
