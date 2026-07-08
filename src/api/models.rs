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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_from_govee_api2() {
        let api_device = govee_api2::Device {
            device: "AA:BB:CC:00:11:22:33:44".into(),
            sku: "H6072".into(),
            device_name: "Floor Lamp".into(),
            device_type: Some("devices.types.light".into()),
            capabilities: vec![
                govee_api2::Capability {
                    capability_type: "devices.capabilities.on_off".into(),
                    instance: "powerSwitch".into(),
                    parameters: serde_json::json!({}),
                },
                govee_api2::Capability {
                    capability_type: "devices.capabilities.range".into(),
                    instance: "brightness".into(),
                    parameters: serde_json::json!({}),
                },
                govee_api2::Capability {
                    capability_type: "devices.capabilities.color_setting".into(),
                    instance: "colorRgb".into(),
                    parameters: serde_json::json!({}),
                },
                govee_api2::Capability {
                    capability_type: "devices.capabilities.color_setting".into(),
                    instance: "colorTemperatureK".into(),
                    parameters: serde_json::json!({}),
                },
            ],
        };

        let device = Device::from(api_device);
        assert_eq!(device.id, "AA:BB:CC:00:11:22:33:44");
        assert_eq!(device.name, "Floor Lamp");
        assert_eq!(device.model, "H6072");
        assert!(device.controllable);
        assert!(device.retrievable);
        assert!(!device.is_group);
        assert!(device.supports_power);
        assert!(device.supports_brightness);
        assert!(device.supports_color);
        assert!(device.supports_color_temp);
        assert!(!device.supports_scenes);
    }

    #[test]
    fn test_device_from_group() {
        let api_device = govee_api2::Device {
            device: "group-1".into(),
            sku: "SameModeGroup".into(),
            device_name: "Downstairs".into(),
            device_type: None,
            capabilities: vec![],
        };

        let device = Device::from(api_device);
        assert!(device.is_group);
        assert!(!device.controllable);
        assert!(!device.retrievable);
    }

    #[test]
    fn test_device_from_device_with_scenes() {
        let api_device = govee_api2::Device {
            device: "id-1".into(),
            sku: "H6072".into(),
            device_name: "Lamp".into(),
            device_type: None,
            capabilities: vec![
                govee_api2::Capability {
                    capability_type: "devices.capabilities.dynamic_scene".into(),
                    instance: "lightScene".into(),
                    parameters: serde_json::json!({}),
                },
            ],
        };

        let device = Device::from(api_device);
        assert!(device.supports_scenes);
    }

    #[test]
    fn test_device_retrievable_requires_sensor() {
        // No brightness or color capabilities → not retrievable
        let api_device = govee_api2::Device {
            device: "id-1".into(),
            sku: "H6072".into(),
            device_name: "Minimal".into(),
            device_type: None,
            capabilities: vec![
                govee_api2::Capability {
                    capability_type: "devices.capabilities.on_off".into(),
                    instance: "powerSwitch".into(),
                    parameters: serde_json::json!({}),
                },
            ],
        };

        let device = Device::from(api_device);
        assert!(!device.retrievable);
    }

    #[test]
    fn test_rgb_color_new() {
        let c = RgbColor::new(100, 150, 200);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
    }

    #[test]
    fn test_rgb_color_serde_round_trip() {
        let c = RgbColor::new(10, 20, 30);
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: RgbColor = serde_json::from_str(&json).unwrap();
        assert_eq!(c.r, deserialized.r);
        assert_eq!(c.g, deserialized.g);
        assert_eq!(c.b, deserialized.b);
    }
}
