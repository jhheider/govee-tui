# govee-api2

A Rust client for Govee's v2 router-based platform API
(`https://openapi.api.govee.com`).

## Features

- Async/await with tokio
- Device and group discovery
- Device state queries
- Power, brightness, RGB color, and color temperature control
- Dynamic light scenes and DIY scenes (list + activate)
- Per-segment color and brightness for segmented lights
- Configurable timeout and retries with exponential backoff
- Typed errors, including rate-limit detection with `Retry-After` parsing
- TLS via rustls (no OpenSSL dependency)

## Installation

```bash
cargo add govee-api2
```

Or, to track unreleased changes, as a git dependency:

```toml
[dependencies]
govee-api2 = { git = "https://github.com/jhheider/govee-tui" }
```

## Usage

```rust,no_run
use govee_api2::{Color, GoveeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GoveeClient::new("your-api-key-here");

    // List all devices
    let devices = client.get_devices().await?;
    for device in &devices {
        println!(
            "{}: {} ({})",
            if device.is_group() { "Group" } else { "Device" },
            device.device_name,
            device.sku
        );
    }

    if let Some(device) = devices.first() {
        // Turn on
        client.turn_on(&device.device, &device.sku).await?;

        // Set brightness (0-100)
        client.set_brightness(&device.device, &device.sku, 80).await?;

        // Set color
        if device.supports_color() {
            let blue = Color::new(0, 0, 255);
            client.set_color(&device.device, &device.sku, blue).await?;
        }

        // Set color temperature (2000-9000K)
        if device.supports_color_temp() {
            client
                .set_color_temperature(&device.device, &device.sku, 4000)
                .await?;
        }

        // Get current state
        let state = client.get_device_state(&device.device, &device.sku).await?;
        println!("Power: {}", state.power);
        println!("Brightness: {:?}", state.brightness);
    }

    Ok(())
}
```

### Configuration

`GoveeClient::new` uses a 10 second timeout and 3 retries. Transport errors
and HTTP 5xx responses are retried with exponential backoff; 429 rate-limit
responses are never retried and surface immediately as
`Error::RateLimited`.

```rust,no_run
use std::time::Duration;
use govee_api2::{ClientConfig, GoveeClient};

let client = GoveeClient::with_config(
    "your-api-key-here",
    ClientConfig {
        timeout: Duration::from_secs(5),
        retry_attempts: 5,
        ..ClientConfig::default()
    },
);
```

### Scenes

```rust,no_run
# use govee_api2::GoveeClient;
# async fn example(client: GoveeClient) -> Result<(), govee_api2::Error> {
# let (device, sku) = ("id", "sku");
// Dynamic light scenes (built-in, e.g. "Sunrise")
let scenes = client.get_dynamic_scenes(device, sku).await?;
if let Some(scene) = scenes.iter().find(|s| s.name == "Sunrise") {
    client.set_scene(device, sku, scene).await?;
}

// DIY scenes (created by the user in the Govee Home app)
let diy = client.get_diy_scenes(device, sku).await?;
if let Some(scene) = diy.first() {
    client.set_scene(device, sku, scene).await?;
}
# Ok(())
# }
```

### Segmented lights

For devices with `devices.capabilities.segment_color_setting` (light strips,
some lamps), individual segments can be colored and dimmed:

```rust,no_run
# use govee_api2::GoveeClient;
# async fn example(client: GoveeClient) -> Result<(), govee_api2::Error> {
# let (device, sku) = ("id", "sku");
// Segments 0-3 red
client.set_segment_color(device, sku, &[0, 1, 2, 3], 255, 0, 0).await?;

// Segments 4-5 at 50% brightness
client.set_segment_brightness(device, sku, &[4, 5], 50).await?;
# Ok(())
# }
```

## API Coverage

This crate implements the following Govee platform API endpoints:

- `GET /router/api/v1/user/devices` - List devices and groups
- `POST /router/api/v1/device/state` - Get device state
- `POST /router/api/v1/device/control` - Control device
- `POST /router/api/v1/device/scenes` - List dynamic light scenes
- `POST /router/api/v1/device/diy-scenes` - List DIY scenes

### Supported Capabilities

- `devices.capabilities.on_off` - Power control
- `devices.capabilities.range` - Brightness control
- `devices.capabilities.color_setting` - RGB color and color temperature
- `devices.capabilities.dynamic_scene` - Dynamic light scenes
- `devices.capabilities.diy_color_setting` - DIY scenes
- `devices.capabilities.segment_color_setting` - Per-segment color/brightness

Other capability types (work modes, toggles, music mode, sensors, ...) are
exposed as raw `Capability` values on `Device` but do not have dedicated
helper methods.

## Device Groups

The API supports device groups (multiple devices controlled together).
Groups have:

- `sku == "SameModeGroup"`
- No `device_type` field
- Same control capabilities as individual devices

```rust,no_run
# use govee_api2::GoveeClient;
# async fn example(client: GoveeClient) -> Result<(), govee_api2::Error> {
let devices = client.get_devices().await?;
for device in devices {
    if device.is_group() {
        println!("Found group: {}", device.device_name);
        // Groups can be controlled just like individual devices
        client.turn_on(&device.device, &device.sku).await?;
    }
}
# Ok(())
# }
```

## Error Handling

All methods return `Result<T, govee_api2::Error>`:

```rust,no_run
# use govee_api2::GoveeClient;
# async fn example(client: GoveeClient) {
use govee_api2::Error;

match client.get_devices().await {
    Ok(devices) => println!("Found {} devices", devices.len()),
    Err(Error::InvalidApiKey) => {
        eprintln!("The API key was rejected");
    }
    Err(Error::RateLimited { retry_after_secs }) => {
        eprintln!("Rate limited; retry after {retry_after_secs:?} seconds");
    }
    Err(Error::Api { code, message }) => {
        eprintln!("API error {}: {}", code, message);
    }
    Err(Error::Request(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => eprintln!("Error: {}", e),
}
# }
```

## Rate Limits

Govee's platform API allows 10,000 requests per account per day and answers
HTTP 429 when exceeded. This client never blind-retries 429 responses; it
returns `Error::RateLimited` with a retry hint parsed from the
`Retry-After` header (or `X-RateLimit-Reset`/`API-RateLimit-Reset`, if
present).

## Getting an API Key

1. Download the Govee Home app
2. Go to Settings → Apply for API Key
3. Follow the email instructions
4. Your API key will be sent via email

## License

MIT OR Apache-2.0
