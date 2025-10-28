# govee-api2

A complete Rust client for Govee's v2 router-based API.

## Features

- ✅ Full async/await support with tokio
- ✅ Device and group discovery
- ✅ Device state queries
- ✅ Power control (on/off)
- ✅ Brightness control (0-100)
- ✅ RGB color control
- ✅ Color temperature control (2000-9000K)
- ✅ Proper error handling with thiserror
- ✅ Support for device groups
- ✅ Type-safe API with builder patterns

## Installation

```toml
[dependencies]
govee-api2 = "0.1"
```

## Usage

```rust
use govee_api2::{GoveeClient, Color};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GoveeClient::new("your-api-key-here");

    // List all devices
    let devices = client.get_devices().await?;
    for device in &devices {
        println!("{}: {} ({})",
            if device.is_group() { "📦 Group" } else { "💡 Device" },
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
        println!("Power: {:?}", state.power_state);
        println!("Brightness: {:?}", state.brightness);
    }

    Ok(())
}
```

## API Coverage

This crate implements the following Govee API v2 endpoints:

- `GET /router/api/v1/user/devices` - List devices and groups
- `POST /router/api/v1/device/state` - Get device state
- `POST /router/api/v1/device/control` - Control device

### Supported Capabilities

- `devices.capabilities.on_off` - Power control
- `devices.capabilities.range` - Brightness control
- `devices.capabilities.color_setting` - RGB color and color temperature

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
