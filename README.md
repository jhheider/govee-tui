# Govee TUI

A clean, colorful terminal user interface (TUI) for controlling Govee smart home devices.

## Features

- 💡 **Device Management**: List, inspect, and control all your Govee devices
- 🎨 **Colorful Interface**: Beautiful emoji-rich TUI with real-time updates
- ⚡ **Full Control**: Power, brightness, RGB color, and color temperature
- 🎨 **Interactive Color Picker**: RGB editor + 150+ named colors browser
- 🌡️ **Color Temperature**: Adjust warm/cool lighting (2000K-9000K)
- 📊 **Device Details**: View comprehensive device state and info
- 🎚️ **Fine-Grained Control**: Shift+arrows for precise adjustments
- 🗄️ **Smart Caching**: SQLite-based device state caching
- ⌨️ **Vim-style Navigation**: Intuitive keyboard shortcuts (hjkl)
- 🚀 **Fast & Efficient**: Static binary with minimal dependencies
- 🖥️ **CLI Mode**: Non-interactive commands for scripting

### Planned Features
- 🔍 **Device Search**: Filter devices by name (Ctrl+F)
- 🎯 **Multi-Device Control**: Select and control multiple devices at once

## Installation

### From Pre-built Binaries

Download the latest release for your platform:

```bash
# Linux (x86_64)
curl -L https://github.com/jhheider/govee-tui/releases/latest/download/govee-tui-linux-x86_64.tar.gz | tar xz
sudo mv govee-tui /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/jhheider/govee-tui/releases/latest/download/govee-tui-macos-x86_64.tar.gz | tar xz
sudo mv govee-tui /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/jhheider/govee-tui/releases/latest/download/govee-tui-macos-aarch64.tar.gz | tar xz
sudo mv govee-tui /usr/local/bin/
```

### From Source

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/jhheider/govee-tui
cd govee-tui
cargo build --release

# Install
sudo cp target/release/govee-tui /usr/local/bin/
```

## Configuration

On first run, a configuration file will be created at `~/.config/govee-tui/config.toml`:

```toml
[api]
key = "YOUR_API_KEY_HERE"  # Get from https://developer.govee.com
timeout_ms = 5000
retry_attempts = 3

[ui]
theme = "dark"
emoji = true
refresh_interval_ms = 5000

[database]
path = "~/.local/share/govee-tui/devices.db"
cache_ttl_seconds = 300
```

### Getting a Govee API Key

1. Download the Govee Home app
2. Go to Settings → About Us → Apply for API Key
3. Follow the instructions to receive your key via email
4. Add the key to your config file

## Usage

### Interactive TUI Mode (default)

```bash
govee-tui
```

**Keybindings:**

#### Global Keys
- `?` - Show help modal
- `r` - Refresh devices
- `Tab` - Switch focus between List and Detail panes
- `q` / `Ctrl+C` - Quit

#### Device List View (when focused)
- `↑/↓` or `j/k` - Navigate device list
- `Enter` - Focus detail pane for selected device

#### Device Detail View
- `Esc` - Back to list
- `Space` - Toggle power ON/OFF
- `↑/↓` or `j/k` - Adjust brightness ±10%
- `Shift+↑/↓` or `J/K` - Adjust brightness ±5% (fine control)
- `c` - Open RGB color picker
- `t` - Decrease color temperature (warmer, -500K)
- `T` - Increase color temperature (cooler, +500K)

#### Color Picker
- `Tab` - Switch between RGB editor and Named Colors browser
- **RGB Mode:**
  - `↑/↓` - Select R/G/B channel
  - `←/→` - Adjust value ±10
- **Browser Mode:**
  - `↑/↓` - Navigate colors within group
  - `←/→` - Switch color group
- `Enter` - Apply color
- `Esc` - Cancel

### CLI Mode

List all devices:
```bash
govee-tui devices
```

Get device status:
```bash
# By name (fuzzy match)
govee-tui status "Living Room"

# By exact ID
govee-tui status "AA:BB:CC:DD:EE:FF:11:22"
```

Control devices:
```bash
# Turn on/off
govee-tui control "Living Room" turn on
govee-tui control "Bedroom" turn off

# Set brightness (0-100)
govee-tui control "Kitchen" brightness 75

# Set RGB color
govee-tui control "Strip" color 255 0 0  # Red

# Set color temperature (2000-9000K)
govee-tui control "Desk Lamp" temp 4000
```

### Direct API Testing (Developer Scripts)

For debugging and testing the Govee API directly without the TUI, use the included bash scripts:

```bash
# Interactive test menu
./scripts/test-api.sh

# List all devices (saves to /tmp/govee-devices.json)
./scripts/get-devices.sh

# Get device state
./scripts/get-device-state.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159"

# Control device
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" on
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" brightness:75
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" color:255,128,0
```

**Requirements:** `curl` and `jq`

See [scripts/README.md](scripts/README.md) for full documentation.

**Use cases:**
- Debug missing devices issue (compare new vs legacy endpoints)
- Test API without building the app
- Verify API key and permissions
- Quick device control from command line
- CI/CD integration testing

## Supported Commands (CLI)

| Command | Description | Range |
|---------|-------------|-------|
| `turn` | Power on/off | `on`, `off` |
| `brightness` | Set brightness | 0-100% |
| `color` | Set RGB color | 0-255 per channel |
| `temp` | Set color temperature | 2000-9000K |

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Project Structure

```
govee-tui/
├── src/
│   ├── main.rs          # Entry point + CLI args
│   ├── config.rs        # Configuration management
│   ├── api/             # Govee API client
│   │   ├── mod.rs       # Client wrapper
│   │   ├── models.rs    # Device models
│   │   └── commands.rs  # Control commands
│   ├── db/              # SQLite persistence
│   │   ├── mod.rs       # Database connection
│   │   ├── schema.rs    # Schema definitions
│   │   └── cache.rs     # Caching layer
│   └── ui/              # TUI interface
│       ├── mod.rs       # App state & event loop
│       ├── theme.rs     # Colors & emojis
│       └── widgets/     # UI components
├── .github/workflows/   # CI/CD pipelines
└── Cargo.toml          # Dependencies
```

## CI/CD

The project uses GitHub Actions for continuous integration:

- **Format Check**: `rustfmt` validation
- **Lint**: `clippy` with strict warnings
- **Test**: Unit and integration tests
- **Build**: Cross-platform compilation (Linux, macOS)
- **Release**: Automated binary releases on git tags
- **Security Audit**: Weekly dependency security checks

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Ensure `cargo fmt` and `cargo clippy` pass
4. Add tests for new features
5. Submit a pull request

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) TUI framework
- Uses the [Govee API v2](https://developer.govee.com/reference/get-you-devices)
- Inspired by the need for a clean terminal interface for smart home control
