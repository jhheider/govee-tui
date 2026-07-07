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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Write-then-rename so a crash mid-write can't leave a truncated file
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(devices)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}
