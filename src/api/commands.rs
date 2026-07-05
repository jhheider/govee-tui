use govee_api2::Scene;

#[derive(Debug, Clone)]
pub enum Command {
    TurnOn,
    TurnOff,
    Brightness(u8),
    Color(u8, u8, u8),
    ColorTemp(u16),
    Scene(Scene),
}

impl Command {
    pub fn turn(on: bool) -> Self {
        if on {
            Self::TurnOn
        } else {
            Self::TurnOff
        }
    }

    pub fn brightness(value: u8) -> Self {
        let clamped = value.min(100);
        Self::Brightness(clamped)
    }

    pub fn color(r: u8, g: u8, b: u8) -> Self {
        Self::Color(r, g, b)
    }

    pub fn temperature(kelvin: u16) -> Self {
        let clamped = kelvin.clamp(2000, 9000);
        Self::ColorTemp(clamped)
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
            Command::ColorTemp(val) => assert_eq!(val, 2000),
            _ => panic!("Wrong command type"),
        }

        match cmd2 {
            Command::ColorTemp(val) => assert_eq!(val, 9000),
            _ => panic!("Wrong command type"),
        }
    }
}
