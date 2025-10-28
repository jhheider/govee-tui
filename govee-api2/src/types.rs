use serde::{Deserialize, Serialize};

/// A Govee device or device group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// Device/group identifier
    pub device: String,

    /// SKU/model identifier
    pub sku: String,

    /// User-friendly device name
    pub device_name: String,

    /// Device type (e.g., "devices.types.light"), None for groups
    #[serde(rename = "type")]
    pub device_type: Option<String>,

    /// List of capabilities this device supports
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

impl Device {
    /// Returns true if this is a device group (not an individual device)
    pub fn is_group(&self) -> bool {
        self.sku == "SameModeGroup"
    }

    /// Returns true if the device supports on/off control
    pub fn supports_power(&self) -> bool {
        self.has_capability("devices.capabilities.on_off")
    }

    /// Returns true if the device supports brightness control
    pub fn supports_brightness(&self) -> bool {
        self.has_capability("devices.capabilities.range")
    }

    /// Returns true if the device supports color control
    pub fn supports_color(&self) -> bool {
        self.has_capability("devices.capabilities.color_setting")
    }

    /// Returns true if the device supports color temperature control
    pub fn supports_color_temp(&self) -> bool {
        self.has_capability("devices.capabilities.color_setting")
    }

    fn has_capability(&self, capability_type: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.capability_type == capability_type)
    }
}

/// A device capability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    /// Capability type (e.g., "devices.capabilities.on_off")
    #[serde(rename = "type")]
    pub capability_type: String,

    /// Instance identifier
    pub instance: String,

    /// Capability-specific parameters
    #[serde(default)]
    pub parameters: serde_json::Value,
}

/// Raw device state response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStateResponse {
    pub capabilities: Vec<CapabilityState>,
}

/// A capability state from the device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityState {
    #[serde(rename = "type")]
    pub capability_type: String,
    pub instance: String,
    pub state: StateValue,
}

/// State value can be an int, object, or other JSON value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    Int { value: i32 },
    Object { value: serde_json::Value },
}

/// Parsed device state information (user-friendly)
#[derive(Debug, Clone)]
pub struct DeviceState {
    /// Power on/off
    pub power: bool,

    /// Brightness (0-100)
    pub brightness: Option<i32>,

    /// RGB color
    pub color: Option<Color>,

    /// Color temperature in Kelvin
    pub color_temperature_kelvin: Option<i32>,
}

impl DeviceState {
    /// Parse from raw capabilities array
    pub fn from_capabilities(capabilities: Vec<CapabilityState>) -> Self {
        let mut power = false;
        let mut brightness = None;
        let mut color = None;
        let mut color_temperature_kelvin = None;

        for cap in capabilities {
            match cap.instance.as_str() {
                "powerSwitch" => {
                    if let StateValue::Int { value } = cap.state {
                        power = value == 1;
                    }
                }
                "brightness" => {
                    if let StateValue::Int { value } = cap.state {
                        brightness = Some(value);
                    }
                }
                "colorRgb" => {
                    if let StateValue::Object { value } = cap.state {
                        if let Ok(rgb) = serde_json::from_value::<Color>(value) {
                            color = Some(rgb);
                        }
                    }
                }
                "colorTemperatureK" | "colorTem" => {
                    if let StateValue::Int { value } = cap.state {
                        color_temperature_kelvin = Some(value);
                    }
                }
                _ => {}
            }
        }

        Self {
            power,
            brightness,
            color,
            color_temperature_kelvin,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerState {
    On,
    Off,
}

impl PowerState {
    pub fn is_on(&self) -> bool {
        matches!(self, PowerState::On)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

// Internal API response types
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub message: String,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub(crate) struct ControlRequest {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub payload: ControlPayload,
}

#[derive(Debug, Serialize)]
pub(crate) struct ControlPayload {
    pub sku: String,
    pub device: String,
    pub capability: CapabilityCommand,
}

#[derive(Debug, Serialize)]
pub(crate) struct CapabilityCommand {
    #[serde(rename = "type")]
    pub capability_type: String,
    pub instance: String,
    pub value: serde_json::Value,
}
