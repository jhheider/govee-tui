use serde_json::json;

#[derive(Debug, Clone)]
pub enum Command {
    Turn(bool),
    Brightness(u8),
    Color { r: u8, g: u8, b: u8 },
    Temperature(u16),
}

impl Command {
    pub fn turn(on: bool) -> Self {
        Self::Turn(on)
    }

    pub fn brightness(value: u8) -> Self {
        let clamped = value.min(100);
        Self::Brightness(clamped)
    }

    pub fn color(r: u8, g: u8, b: u8) -> Self {
        Self::Color { r, g, b }
    }

    pub fn temperature(kelvin: u16) -> Self {
        let clamped = kelvin.clamp(2000, 9000);
        Self::Temperature(clamped)
    }

    /// Convert command to Govee API GoveeCommand
    pub fn to_govee_command(&self) -> govee_api::structs::govee::GoveeCommand {
        match self {
            Self::Turn(on) => {
                let value = if *on { "on" } else { "off" };
                govee_api::structs::govee::GoveeCommand {
                    name: "turn".to_string(),
                    value: value.to_string(),
                }
            }
            Self::Brightness(val) => govee_api::structs::govee::GoveeCommand {
                name: "brightness".to_string(),
                value: val.to_string(),
            },
            Self::Color { r, g, b } => govee_api::structs::govee::GoveeCommand {
                name: "color".to_string(),
                value: json!({"r": r, "g": g, "b": b}).to_string(),
            },
            Self::Temperature(kelvin) => govee_api::structs::govee::GoveeCommand {
                name: "colorTem".to_string(),
                value: kelvin.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_clamping() {
        let cmd = Command::brightness(150);
        match cmd {
            Command::Brightness(val) => assert_eq!(val, 100),
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_temperature_clamping() {
        let cmd1 = Command::temperature(1000);
        let cmd2 = Command::temperature(10000);

        match cmd1 {
            Command::Temperature(val) => assert_eq!(val, 2000),
            _ => panic!("Wrong command type"),
        }

        match cmd2 {
            Command::Temperature(val) => assert_eq!(val, 9000),
            _ => panic!("Wrong command type"),
        }
    }
}
