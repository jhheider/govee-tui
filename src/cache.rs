use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::api::Device;

fn cache_file() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("govee-tui").join("devices.json"))
}

/// Last-seen device list, used to paint the UI before the first API response
pub fn load_devices() -> Option<Vec<Device>> {
    let content = std::fs::read_to_string(cache_file()?).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_devices(devices: &[Device]) -> Result<()> {
    let Some(path) = cache_file() else {
        return Ok(());
    };
    atomic_save(devices, &path)
}

/// Write-then-rename so a crash mid-write can't leave a truncated file.
pub(crate) fn atomic_save(devices: &[Device], path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(devices)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Device;

    fn test_device(name: &str) -> Device {
        Device {
            id: format!("test-{name}"),
            name: name.into(),
            model: "H6072".into(),
            controllable: true,
            retrievable: true,
            is_group: false,
            device_type: Some("devices.types.light".into()),
            supports_power: true,
            supports_brightness: true,
            supports_color: true,
            supports_color_temp: true,
            supports_scenes: false,
        }
    }

    #[test]
    fn atomic_save_creates_file() {
        let dir = std::env::temp_dir().join("govee-tui-test-atomic-save");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("devices.json");

        let devices = vec![test_device("Lamp")];
        atomic_save(&devices, &path).unwrap();

        assert!(path.exists());
        let loaded: Vec<Device> = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Lamp");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_save_round_trip_preserves_fields() {
        let dir = std::env::temp_dir().join("govee-tui-test-round-trip");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("devices.json");

        let device = test_device("Desk");
        let devices = vec![device.clone()];
        atomic_save(&devices, &path).unwrap();

        let loaded: Vec<Device> = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        assert_eq!(loaded[0].id, device.id);
        assert_eq!(loaded[0].name, device.name);
        assert_eq!(loaded[0].model, device.model);
        assert_eq!(loaded[0].controllable, device.controllable);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_save_overwrites_existing_file() {
        let dir = std::env::temp_dir().join("govee-tui-test-overwrite");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("devices.json");

        let original = vec![test_device("Original")];
        atomic_save(&original, &path).unwrap();

        let replacement = vec![test_device("Replacement")];
        atomic_save(&replacement, &path).unwrap();

        let loaded: Vec<Device> = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Replacement");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_save_empty_list() {
        let dir = std::env::temp_dir().join("govee-tui-test-empty");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("devices.json");

        atomic_save(&[], &path).unwrap();
        let loaded: Vec<Device> = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        assert!(loaded.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
