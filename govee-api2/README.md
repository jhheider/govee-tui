# govee-api2

A complete Rust client for Govee's v2 router-based API.

## Features

- Full async/await support with tokio
- Device and group discovery
- Device state queries
- Power control (on/off)
- Brightness control (0-100)
- RGB color control
- Color temperature control (2000-9000K)
- Dynamic scenes and DIY scenes
- Segment color control for RGBIC devices
- Toggle controls (gradient, nightlight)
- Proper error handling with thiserror
- Support for device groups
- Type-safe API with builder patterns
- Configurable TLS backend (rustls or native-tls)

## Installation

```toml
[dependencies]
govee-api2 = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### TLS Backend Selection

By default uses `rustls` (pure Rust, easier cross-compilation). To use native TLS:

```toml
[dependencies]
govee-api2 = { version = "0.1", default-features = false, features = ["native-tls"] }
```

## Usage

### Basic Usage

```rust
use govee_api2::{GoveeClient, Color};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GoveeClient::new("your-api-key-here");

    // List all devices
    let devices = client.get_devices().await?;
    for device in &devices {
        println!("{}: {} ({})",
            if device.is_group() { "Group" } else { "Device" },
            device.device_name,
            device.sku
        );
    }

    // Control a device
    if let Some(device) = devices.first() {
        // Turn on
        client.turn_on(&device.device, &device.sku).await?;

        // Set brightness
        client.set_brightness(&device.device, &device.sku, 80).await?;

        // Set color
        let blue = Color::new(0, 0, 255);
        client.set_color(&device.device, &device.sku, blue).await?;

        // Set color temperature
        client.set_color_temperature(&device.device, &device.sku, 4000).await?;

        // Get current state
        let state = client.get_device_state(&device.device, &device.sku).await?;
        println!("Power: {}", state.power);
        println!("Brightness: {:?}", state.brightness);
    }

    Ok(())
}
```

### Builder Pattern

Use the builder for custom client configuration:

```rust
use govee_api2::GoveeClient;
use std::time::Duration;

let client = GoveeClient::builder("your-api-key")
    .timeout(Duration::from_secs(10))
    .user_agent("my-app/1.0")
    .build()
    .expect("Failed to build client");
```

### Dynamic Scenes

```rust
// Apply a dynamic scene by ID
client.set_dynamic_scene(&device.device, &device.sku, 123).await?;

// Apply a user-created DIY scene
client.set_diy_scene(&device.device, &device.sku, 456).await?;
```

### RGBIC Segment Control

```rust
use govee_api2::Color;

// Set individual segment colors on addressable LED strips
client.set_segment_colors(&device.device, &device.sku, &[
    (0, Color::new(255, 0, 0)),   // Segment 0: Red
    (1, Color::new(0, 255, 0)),   // Segment 1: Green
    (2, Color::new(0, 0, 255)),   // Segment 2: Blue
]).await?;
```

### Toggle Controls

```rust
// Enable gradient mode
client.set_toggle(&device.device, &device.sku, "gradientToggle", true).await?;

// Enable nightlight mode
client.set_toggle(&device.device, &device.sku, "nightlightToggle", true).await?;
```

## API Coverage

This crate implements the following Govee API v2 endpoints:

- `GET /router/api/v1/user/devices` - List devices and groups
- `POST /router/api/v1/device/state` - Get device state
- `POST /router/api/v1/device/control` - Control device

### Supported Capabilities

| Capability | Methods |
|------------|---------|
| `devices.capabilities.on_off` | `turn_on()`, `turn_off()` |
| `devices.capabilities.range` | `set_brightness()` |
| `devices.capabilities.color_setting` | `set_color()`, `set_color_temperature()` |
| `devices.capabilities.dynamic_scene` | `set_dynamic_scene()`, `set_diy_scene()` |
| `devices.capabilities.segment_color_setting` | `set_segment_colors()` |
| `devices.capabilities.toggle` | `set_toggle()` |

## Device Groups

The API supports device groups (multiple devices controlled together). Groups have:
- `sku == "SameModeGroup"`
- No `device_type` field
- Same control capabilities as individual devices

```rust
let devices = client.get_devices().await?;
for device in devices {
    if device.is_group() {
        println!("Found group: {}", device.device_name);
        // Groups can be controlled just like individual devices
        client.turn_on(&device.device, &device.sku).await?;
    }
}
```

## Error Handling

All methods return `Result<T, govee_api2::Error>`:

```rust
use govee_api2::Error;

match client.get_devices().await {
    Ok(devices) => println!("Found {} devices", devices.len()),
    Err(Error::Api { code, message }) => {
        eprintln!("API error {}: {}", code, message);
    }
    Err(Error::Request(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Getting an API Key

1. Download the Govee Home app
2. Go to Settings → About Us → Apply for API Key
3. Follow the email instructions
4. Your API key will be sent via email

## License

MIT OR Apache-2.0
