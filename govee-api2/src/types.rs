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
        self.has_instance("devices.capabilities.range", "brightness")
    }

    /// Returns true if the device supports RGB color control
    pub fn supports_color(&self) -> bool {
        self.has_instance("devices.capabilities.color_setting", "colorRgb")
    }

    /// Returns true if the device supports color temperature control
    pub fn supports_color_temp(&self) -> bool {
        self.has_instance("devices.capabilities.color_setting", "colorTemperatureK")
    }

    /// Returns true if the device supports dynamic light scenes
    pub fn supports_scenes(&self) -> bool {
        self.has_capability("devices.capabilities.dynamic_scene")
    }

    /// Returns true if the device supports per-segment color/brightness control
    pub fn supports_segments(&self) -> bool {
        self.has_capability("devices.capabilities.segment_color_setting")
    }

    fn has_capability(&self, capability_type: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.capability_type == capability_type)
    }

    fn has_instance(&self, capability_type: &str, instance: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.capability_type == capability_type && c.instance == instance)
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

/// A light scene (dynamic or DIY) that can be activated on a device.
///
/// Dynamic light scenes (from `/router/api/v1/device/scenes`) carry a
/// `param_id` alongside the scene `id`; DIY scenes (from
/// `/router/api/v1/device/diy-scenes`) are identified by a single integer
/// value, so `param_id` is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    /// Human-readable scene name (e.g., "Sunrise")
    pub name: String,

    /// Scene identifier sent in the control request
    pub id: i64,

    /// Companion parameter id (dynamic light scenes only)
    pub param_id: Option<i64>,

    /// Capability type to send when activating this scene
    /// (e.g., "devices.capabilities.dynamic_scene" or
    /// "devices.capabilities.diy_color_setting")
    pub capability_type: String,

    /// Capability instance to send when activating this scene
    /// (e.g., "lightScene" or "diyScene")
    pub instance: String,
}

impl Scene {
    /// The control-request value for this scene:
    /// `{"paramId": .., "id": ..}` for dynamic scenes, a bare integer for
    /// DIY scenes.
    pub fn control_value(&self) -> serde_json::Value {
        match self.param_id {
            Some(param_id) => serde_json::json!({ "paramId": param_id, "id": self.id }),
            None => serde_json::json!(self.id),
        }
    }

    /// Extract scenes from a list of scene capabilities (as returned by the
    /// scenes and diy-scenes endpoints).
    pub fn from_capabilities(capabilities: &[Capability]) -> Vec<Scene> {
        let mut scenes = Vec::new();

        for cap in capabilities {
            let Some(options) = cap.parameters.get("options").and_then(|o| o.as_array()) else {
                continue;
            };

            for option in options {
                let Some(name) = option.get("name").and_then(|n| n.as_str()) else {
                    continue;
                };
                let Some(value) = option.get("value") else {
                    continue;
                };

                let (id, param_id) = if let Some(id) = value.as_i64() {
                    (id, None)
                } else if let Some(id) = value.get("id").and_then(|v| v.as_i64()) {
                    (id, value.get("paramId").and_then(|v| v.as_i64()))
                } else {
                    continue;
                };

                scenes.push(Scene {
                    name: name.to_string(),
                    id,
                    param_id,
                    capability_type: cap.capability_type.clone(),
                    instance: cap.instance.clone(),
                });
            }
        }

        scenes
    }
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

    /// Whether the device is reachable over wifi (if reported)
    pub online: Option<bool>,

    /// Brightness (0-100)
    pub brightness: Option<i32>,

    /// RGB color
    pub color: Option<Color>,

    /// Color temperature in Kelvin
    pub color_temperature_kelvin: Option<i32>,

    /// Active dynamic light scene id (if the device reports one)
    pub light_scene: Option<i64>,

    /// Active DIY scene id (if the device reports one)
    pub diy_scene: Option<i64>,

    /// Whether the device reports segmented color control in its state
    pub has_segments: bool,
}

impl DeviceState {
    /// Parse from raw capabilities array
    pub fn from_capabilities(capabilities: Vec<CapabilityState>) -> Self {
        let mut power = false;
        let mut online = None;
        let mut brightness = None;
        let mut color = None;
        let mut color_temperature_kelvin = None;
        let mut light_scene = None;
        let mut diy_scene = None;
        let mut has_segments = false;

        for cap in capabilities {
            if cap.capability_type == "devices.capabilities.segment_color_setting" {
                has_segments = true;
                continue;
            }

            match cap.instance.as_str() {
                "powerSwitch" => match cap.state {
                    StateValue::Int { value } => power = value == 1,
                    StateValue::Bool { value } => power = value,
                    _ => {}
                },
                "online" => match cap.state {
                    StateValue::Bool { value } => online = Some(value),
                    StateValue::Int { value } => online = Some(value == 1),
                    _ => {}
                },
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
                "lightScene" => {
                    if let StateValue::Int { value } = cap.state {
                        light_scene = Some(value);
                    }
                }
                "diyScene" => {
                    if let StateValue::Int { value } = cap.state {
                        diy_scene = Some(value);
                    }
                }
                _ => {}
            }
        }

        Self {
            power,
            online,
            brightness,
            color,
            color_temperature_kelvin,
            light_scene,
            diy_scene,
            has_segments,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Pack into the single integer the API expects: (r << 16) | (g << 8) | b
    pub fn to_packed(&self) -> i64 {
        ((self.r as i64) << 16) | ((self.g as i64) << 8) | (self.b as i64)
    }
}

// Internal API response types
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    #[serde(default)]
    pub code: i32,
    #[serde(default, alias = "msg")]
    pub message: String,
    pub data: T,
}

// Device state uses "payload" instead of "data"
#[derive(Debug, Deserialize)]
pub(crate) struct DeviceStateResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default, alias = "message")]
    pub msg: String,
    pub payload: DeviceStatePayload,
}

// Scene list responses (/device/scenes and /device/diy-scenes) wrap
// capabilities in a payload like the state endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct ScenesResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default, alias = "message")]
    pub msg: String,
    pub payload: ScenesPayload,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ScenesPayload {
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

// Device control response has "msg" and "capability" at top level
#[derive(Debug, Deserialize)]
pub(crate) struct ControlResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default, alias = "message")]
    pub msg: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn device_from_json(json: serde_json::Value) -> Device {
        serde_json::from_value(json).unwrap()
    }

    /// Capability list modeled on the H6072 example in Govee's
    /// "Get You Devices" reference documentation.
    fn full_featured_device() -> Device {
        device_from_json(serde_json::json!({
            "sku": "H6072",
            "device": "9D:FA:85:EB:D3:00:8B:FF",
            "deviceName": "Floor Lamp",
            "type": "devices.types.light",
            "capabilities": [
                { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
                  "parameters": { "dataType": "ENUM" } },
                { "type": "devices.capabilities.range", "instance": "brightness",
                  "parameters": { "dataType": "INTEGER", "range": { "min": 1, "max": 100, "precision": 1 } } },
                { "type": "devices.capabilities.segment_color_setting", "instance": "segmentedColorRgb",
                  "parameters": { "dataType": "STRUCT" } },
                { "type": "devices.capabilities.color_setting", "instance": "colorRgb",
                  "parameters": { "dataType": "INTEGER" } },
                { "type": "devices.capabilities.color_setting", "instance": "colorTemperatureK",
                  "parameters": { "dataType": "INTEGER", "range": { "min": 2000, "max": 9000, "precision": 1 } } },
                { "type": "devices.capabilities.dynamic_scene", "instance": "lightScene",
                  "parameters": { "dataType": "ENUM" } }
            ]
        }))
    }

    /// A color-only bulb: has colorRgb but no colorTemperatureK, no scenes,
    /// no segments.
    fn rgb_only_device() -> Device {
        device_from_json(serde_json::json!({
            "sku": "H6008",
            "device": "AA:BB:CC:DD:EE:FF:00:11",
            "deviceName": "Bulb",
            "type": "devices.types.light",
            "capabilities": [
                { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
                  "parameters": { "dataType": "ENUM" } },
                { "type": "devices.capabilities.range", "instance": "brightness",
                  "parameters": { "dataType": "INTEGER" } },
                { "type": "devices.capabilities.color_setting", "instance": "colorRgb",
                  "parameters": { "dataType": "INTEGER" } }
            ]
        }))
    }

    #[test]
    fn capability_detection_full_featured() {
        let device = full_featured_device();
        assert!(!device.is_group());
        assert!(device.supports_power());
        assert!(device.supports_brightness());
        assert!(device.supports_color());
        assert!(device.supports_color_temp());
        assert!(device.supports_scenes());
        assert!(device.supports_segments());
    }

    #[test]
    fn color_temp_requires_color_temperature_instance() {
        // Regression test: colorRgb alone must not imply color temperature.
        let device = rgb_only_device();
        assert!(device.supports_color());
        assert!(!device.supports_color_temp());
        assert!(!device.supports_scenes());
        assert!(!device.supports_segments());
    }

    #[test]
    fn group_detection() {
        let group = device_from_json(serde_json::json!({
            "sku": "SameModeGroup",
            "device": "group-1",
            "deviceName": "Living Room",
            "type": null,
            "capabilities": []
        }));
        assert!(group.is_group());
    }

    #[test]
    fn device_state_from_realistic_capabilities() {
        // Modeled on the "Get Device State" response example in the docs.
        let capabilities: Vec<CapabilityState> = serde_json::from_value(serde_json::json!([
            { "type": "devices.capabilities.online", "instance": "online",
              "state": { "value": true } },
            { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
              "state": { "value": 1 } },
            { "type": "devices.capabilities.range", "instance": "brightness",
              "state": { "value": 42 } },
            { "type": "devices.capabilities.color_setting", "instance": "colorRgb",
              "state": { "value": 16711935 } },
            { "type": "devices.capabilities.color_setting", "instance": "colorTemperatureK",
              "state": { "value": 4000 } },
            { "type": "devices.capabilities.dynamic_scene", "instance": "lightScene",
              "state": { "value": 3853 } },
            { "type": "devices.capabilities.segment_color_setting", "instance": "segmentedColorRgb",
              "state": { "value": "" } }
        ]))
        .unwrap();

        let state = DeviceState::from_capabilities(capabilities);
        assert!(state.power);
        assert_eq!(state.online, Some(true));
        assert_eq!(state.brightness, Some(42));
        let color = state.color.unwrap();
        assert_eq!((color.r, color.g, color.b), (255, 0, 255));
        assert_eq!(state.color_temperature_kelvin, Some(4000));
        assert_eq!(state.light_scene, Some(3853));
        assert_eq!(state.diy_scene, None);
        assert!(state.has_segments);
    }

    #[test]
    fn device_state_handles_empty_and_missing_values() {
        // The docs note that an empty value means "instance does not
        // support query"; make sure those don't break parsing.
        let capabilities: Vec<CapabilityState> = serde_json::from_value(serde_json::json!([
            { "type": "devices.capabilities.online", "instance": "online",
              "state": { "value": false } },
            { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
              "state": { "value": 0 } },
            { "type": "devices.capabilities.range", "instance": "brightness",
              "state": { "value": "" } }
        ]))
        .unwrap();

        let state = DeviceState::from_capabilities(capabilities);
        assert!(!state.power);
        assert_eq!(state.online, Some(false));
        assert_eq!(state.brightness, None);
        assert_eq!(state.color, None);
        assert_eq!(state.color_temperature_kelvin, None);
        assert!(!state.has_segments);
    }

    #[test]
    fn scenes_from_dynamic_scene_capabilities() {
        // Shape from the "Get Dynamic Scene" docs: value is {paramId, id}.
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.dynamic_scene",
                "instance": "lightScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "name": "Sunrise", "value": { "paramId": 4280, "id": 3853 } },
                        { "name": "Sunset", "value": { "paramId": 4281, "id": 3854 } }
                    ]
                }
            }
        ]))
        .unwrap();

        let scenes = Scene::from_capabilities(&capabilities);
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].name, "Sunrise");
        assert_eq!(scenes[0].id, 3853);
        assert_eq!(scenes[0].param_id, Some(4280));
        assert_eq!(
            scenes[0].capability_type,
            "devices.capabilities.dynamic_scene"
        );
        assert_eq!(scenes[0].instance, "lightScene");
        assert_eq!(
            scenes[0].control_value(),
            serde_json::json!({ "paramId": 4280, "id": 3853 })
        );
    }

    #[test]
    fn scenes_from_diy_capabilities() {
        // Shape from the DIY section of the docs: value is a bare integer
        // and the capability type is diy_color_setting.
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.diy_color_setting",
                "instance": "diyScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "name": "Xmas lights 2", "value": 8216931 },
                        { "name": "test", "value": 8216643 }
                    ]
                }
            }
        ]))
        .unwrap();

        let scenes = Scene::from_capabilities(&capabilities);
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].name, "Xmas lights 2");
        assert_eq!(scenes[0].id, 8216931);
        assert_eq!(scenes[0].param_id, None);
        assert_eq!(
            scenes[0].capability_type,
            "devices.capabilities.diy_color_setting"
        );
        assert_eq!(scenes[0].instance, "diyScene");
        assert_eq!(scenes[0].control_value(), serde_json::json!(8216931));
    }

    #[test]
    fn scenes_skip_capabilities_without_options() {
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            { "type": "devices.capabilities.diy_color_setting", "instance": "diyScene",
              "parameters": {} }
        ]))
        .unwrap();
        assert!(Scene::from_capabilities(&capabilities).is_empty());
    }

    #[test]
    fn color_packing() {
        assert_eq!(Color::new(255, 0, 255).to_packed(), 16711935);
        assert_eq!(Color::new(0, 0, 255).to_packed(), 255);
        assert_eq!(Color::new(255, 255, 255).to_packed(), 16777215);
        assert_eq!(Color::new(1, 2, 3).to_hex(), "#010203");
    }

    #[test]
    fn scenes_skip_option_without_name() {
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.dynamic_scene",
                "instance": "lightScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "value": { "paramId": 4280, "id": 3853 } }
                    ]
                }
            }
        ]))
        .unwrap();
        assert!(Scene::from_capabilities(&capabilities).is_empty());
    }

    #[test]
    fn scenes_skip_option_without_value() {
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.dynamic_scene",
                "instance": "lightScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "name": "No Value Here" }
                    ]
                }
            }
        ]))
        .unwrap();
        assert!(Scene::from_capabilities(&capabilities).is_empty());
    }

    #[test]
    fn scenes_skip_option_with_unparseable_value() {
        // value is an object but has neither 'id' nor a bare integer
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.dynamic_scene",
                "instance": "lightScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "name": "Broken", "value": { "notId": 1, "notParamId": 2 } }
                    ]
                }
            }
        ]))
        .unwrap();
        assert!(Scene::from_capabilities(&capabilities).is_empty());
    }

    #[test]
    fn device_state_empty_capabilities() {
        let state = DeviceState::from_capabilities(vec![]);
        assert!(!state.power);
        assert_eq!(state.online, None);
        assert_eq!(state.brightness, None);
        assert!(!state.has_segments);
    }

    #[test]
    fn device_state_unknown_capability_is_ignored() {
        let capabilities: Vec<CapabilityState> = serde_json::from_value(serde_json::json!([
            { "type": "unknown.vendor.type", "instance": "foo",
              "state": { "value": 42 } }
        ]))
        .unwrap();
        // Unknown capabilities should be silently ignored
        let state = DeviceState::from_capabilities(capabilities);
        assert!(!state.power);
        assert_eq!(state.brightness, None);
    }

    #[test]
    fn device_state_handles_color_temperature_old_name() {
        // Some devices report instance="colorTem" instead of "colorTemperatureK"
        let capabilities: Vec<CapabilityState> = serde_json::from_value(serde_json::json!([
            { "type": "devices.capabilities.color_setting", "instance": "colorTem",
              "state": { "value": 3500 } }
        ]))
        .unwrap();
        let state = DeviceState::from_capabilities(capabilities);
        assert_eq!(state.color_temperature_kelvin, Some(3500));
    }

    #[test]
    fn power_state_on_off() {
        assert!(PowerState::On.is_on());
        assert!(!PowerState::Off.is_on());
    }

    #[test]
    fn device_support_methods_no_capabilities() {
        let device = Device {
            device: "test".into(),
            sku: "H1234".into(),
            device_name: "Test".into(),
            device_type: None,
            capabilities: vec![],
        };
        assert!(!device.is_group());
        assert!(!device.supports_power());
        assert!(!device.supports_brightness());
        assert!(!device.supports_color());
        assert!(!device.supports_color_temp());
        assert!(!device.supports_scenes());
        assert!(!device.supports_segments());
    }

    #[test]
    fn device_serde_round_trip() {
        let device = full_featured_device();
        let json = serde_json::to_value(&device).unwrap();
        let deserialized: Device = serde_json::from_value(json).unwrap();
        assert_eq!(device.device, deserialized.device);
        assert_eq!(device.sku, deserialized.sku);
        assert_eq!(device.device_name, deserialized.device_name);
    }

    #[test]
    fn scene_from_capability_missing_parameters() {
        let capabilities: Vec<Capability> = serde_json::from_value(serde_json::json!([
            {
                "type": "devices.capabilities.diy_color_setting",
                "instance": "diyScene",
                "parameters": {
                    "dataType": "ENUM",
                    "options": [
                        { "name": "Valid", "value": 12345 }
                    ]
                }
            }
        ]))
        .unwrap();
        let scenes = Scene::from_capabilities(&capabilities);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].control_value(), serde_json::json!(12345));
    }
}
