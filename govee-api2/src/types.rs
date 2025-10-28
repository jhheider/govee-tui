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

/// Raw device state response from API (wrapped in payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatePayload {
    pub sku: String,
    pub device: String,
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

/// State value can be bool, int, string, or object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    Bool { value: bool },
    Int { value: i64 },
    String { value: String },
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
                        brightness = Some(value as i32);
                    }
                }
                "colorRgb" => {
                    // colorRgb is a single integer packed as: (r << 16) | (g << 8) | b
                    if let StateValue::Int { value } = cap.state {
                        let r = ((value >> 16) & 0xFF) as u8;
                        let g = ((value >> 8) & 0xFF) as u8;
                        let b = (value & 0xFF) as u8;
                        color = Some(Color { r, g, b });
                    }
                }
                "colorTemperatureK" | "colorTem" => {
                    if let StateValue::Int { value } = cap.state {
                        color_temperature_kelvin = Some(value as i32);
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

// Device state uses "payload" instead of "data"
#[derive(Debug, Deserialize)]
pub(crate) struct DeviceStateResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub msg: String,
    pub payload: DeviceStatePayload,
}

// Device control response has "msg" and "capability" at top level
#[derive(Debug, Deserialize)]
pub(crate) struct ControlResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub capability: Option<serde_json::Value>,
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
