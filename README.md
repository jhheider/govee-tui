# Govee TUI

A clean, colorful terminal user interface (TUI) for controlling Govee smart home devices.

## Features

- 💡 **Device Management**: List, inspect, and control all your Govee devices
- 🎨 **Colorful Interface**: Beautiful emoji-rich TUI with real-time updates
- ⚡ **Full Control**: Power, brightness, RGB color, and temperature control
- 🔍 **Device Search**: Quick filter with Ctrl+F
- 🎯 **Multi-Device Control**: Select and control multiple devices at once
- 🎨 **Interactive Color Picker**: RGB selector with real-time preview
- 📊 **Device Details**: View comprehensive device state and info
- 🎚️ **Fine-Grained Control**: Shift+arrows for precise adjustments
- 🗄️ **Smart Caching**: SQLite-based device state caching
- ⌨️ **Vim-style Navigation**: Intuitive keyboard shortcuts
- 🚀 **Fast & Efficient**: Static binary with minimal dependencies

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

#### Device List View
- `↑/k` / `↓/j` - Navigate device list
- `Enter` - View device details
- `Space` - Multi-select device (checkboxes)
- `x` - Clear all selections
- `Ctrl+F` - Search/filter devices
- `r` - Refresh devices
- `q` / `Ctrl+C` - Quit

#### Device Detail View
- `Esc` - Back to list
- `Space` - Toggle power
- `b` - Brightness control
- `c` - RGB color picker
- `t` - Color temperature (planned)

#### Brightness Control
- `↑↓` - Adjust ±5%
- `Shift+↑↓` - Adjust ±1% (fine-grained)
- `1-9` - Set to 10-90%
- `Enter` - Apply changes
- `Esc` - Cancel

#### Color Picker
- `Tab` / `Shift+Tab` - Switch R/G/B channel
- `↑↓` - Adjust ±5
- `Shift+↑↓` - Adjust ±1 (fine-grained)
- `Enter` - Apply color
- `Esc` - Cancel

#### Search Mode
- Type to filter devices by name/model
- `Enter` - Return to list with filter applied
- `Esc` - Cancel search

### CLI Mode

List all devices:
```bash
govee-tui list
```

Control devices:
```bash
# Turn on/off
govee-tui control <device-id> turn on
govee-tui control <device-id> turn off

# Set brightness (0-100)
govee-tui control <device-id> brightness 75

# Set RGB color
govee-tui control <device-id> color 255 0 0  # Red

# Set color temperature (2000-9000K)
govee-tui control <device-id> temp 4000
```

## Supported Commands

| Command | Description | Range | Multi-Device |
|---------|-------------|-------|--------------|
| `turn` | Power on/off | `on`, `off` | ✓ |
| `brightness` | Set brightness | 0-100% | ✓ |
| `color` | Set RGB color | 0-255 per channel | ✓ |
| `temp` | Set color temperature | 2000-9000K | Planned |

### Multi-Device Operations

You can select multiple devices with `Space` and apply commands to all selected devices:
1. Navigate to a device and press `Space` to select (✓ appears in checkbox)
2. Repeat for additional devices
3. Press `Enter` to view details of current device
4. Make changes (brightness, color, power) - applies to ALL selected devices
5. Press `x` to clear all selections

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

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Uses [govee-api](https://github.com/mgierada/govee) Rust crate
- Inspired by the need for a clean terminal interface for smart home control
