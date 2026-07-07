# Govee TUI

[![CI](https://github.com/jhheider/govee-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/jhheider/govee-tui/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A clean, colorful terminal UI for controlling your Govee smart lights - power, brightness, color, temperature, and scenes, all without leaving the keyboard. It's the only actively maintained Govee TUI around.

![demo](demo.gif)

> **How it works:** govee-tui drives Govee's cloud API, so each control is an HTTPS round-trip (a few hundred ms) and counts against Govee's daily rate limit (10,000 requests/day). The app debounces and serializes commands, so normal use stays well within it. Local LAN control is on the [roadmap](#roadmap).

## Installation

### Homebrew (recommended)

```bash
brew install jhheider/tap/govee-tui
```

### Prebuilt binaries

Download a binary from the [latest release](https://github.com/jhheider/govee-tui/releases/latest), extract it, and put `govee-tui` somewhere on your `PATH`. Today's builds cover static musl Linux (x86_64) and native macOS (Intel and Apple Silicon), with more platforms on the way.

### From source

Requires a recent Rust toolchain. govee-tui isn't on crates.io yet, so install straight from the repo:

```bash
cargo install --git https://github.com/jhheider/govee-tui
```

### pkgx (coming soon)

```bash
pkgx govee-tui
```

A pantry PR is in flight; this will work once it lands.

> **You'll need a Govee API key.** Before your first run, either set `GOVEE_API_KEY` or add your key to the config file - see [Getting a Govee API Key](#getting-a-govee-api-key). Controls fail without one.

## Configuration

On first run, a configuration file is created at `~/.config/govee-tui/config.toml` (Linux) or `~/Library/Application Support/govee-tui/config.toml` (macOS):

```toml
[api]
key = "YOUR_API_KEY_HERE"  # Get from the Govee Home app (see below)
timeout_ms = 10000         # Per-request timeout
retry_attempts = 3         # Retries for transport errors / 5xx (never 429)

[ui]
refresh_interval_ms = 30000  # Device-list auto-refresh (minimum 10000)
```

The `GOVEE_API_KEY` environment variable takes precedence over the config file - handy for scripts and CI.

### Getting a Govee API Key

1. Download the Govee Home app
2. Go to Settings -> About Us -> Apply for API Key
3. Follow the instructions to receive your key via email (this can take a little while - it's an application, not an instant token)
4. Add the key to your config file, or export it as `GOVEE_API_KEY`

## Features

- 💡 **Device management**: list, inspect, and control all your Govee devices
- ⚡ **Full control**: power, brightness, RGB color, color temperature, and scenes - all in the TUI (and most via CLI)
- 🎬 **Scene picker**: browse and apply your device's light scenes and DIY scenes
- 🎨 **Interactive color picker**: RGB editor and named-color browser with true-color swatches
- 🚦 **Rate-limit aware**: optimistic updates with debounced sends - hold the brightness key without burning your API budget
- 🗂️ **Instant startup**: paints your last-seen device list from a local cache while it refreshes
- ⌨️ **Vim-style navigation**: `j/k/h/l` plus arrows everywhere
- 🚀 **Small and self-contained**: single binary, rustls (no OpenSSL)

## Usage

### Interactive TUI Mode (default)

```bash
govee-tui
```

**Keybindings:**

#### Global
- `Tab` - Switch focus between device list and detail pane
- `r` - Refresh devices
- `?` - Show/hide help modal
- `q` / `Ctrl+C` - Quit

#### Device List (when focused)
- `↑/k` / `↓/j` - Navigate device list
- `Space` - Toggle power on the selected device
- `Enter` - Focus detail pane

#### Device Detail (when focused)
- `Esc` - Back to list
- `Space` - Toggle power
- `↑/k` / `↓/j` - Brightness ±10%
- `Shift+↑/↓` (or `K` / `J`) - Brightness ±5% (fine-grained)
- `←/h` / `→/l` - Color temperature ±500K (warm <- -> cool)
- `Shift+←/→` (or `H` / `L`) - Color temperature ±100K (fine-grained)
- `c` - Open color picker
- `s` - Open scene picker

#### Color Picker
- `Tab` - Toggle between RGB editor and named-color browser
- RGB mode: `↑/↓` (or `k/j`) switch channel, `←/→` (or `h/l`) adjust ±10
- Browser mode: `↑/↓` navigate colors, `←/→` switch color group
- `Enter` - Apply color
- `Esc` - Cancel

#### Scene Picker
- `↑/k` / `↓/j` - Browse scenes (DIY scenes are tagged)
- `PgUp/PgDn`, `Home/End` - Jump through long scene lists
- `Enter` - Apply scene
- `Esc` - Close

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

# List and apply light scenes (including your DIY scenes)
govee-tui scenes "Floor Lamp"
govee-tui control "Floor Lamp" scene "Movie"
```

### Direct API Testing (Developer Scripts)

For debugging and testing the Govee API directly without the TUI, use the included bash scripts (require `curl` and `jq`):

```bash
./scripts/test-api.sh                 # Interactive test menu
./scripts/get-devices.sh              # List all devices
./scripts/get-device-state.sh <id> <sku>
./scripts/control-device.sh <id> <sku> on|brightness:75|color:255,128,0
```

See [scripts/README.md](scripts/README.md) for full documentation.

## Supported Commands

| Command | Description | Range | TUI | CLI |
|---------|-------------|-------|-----|-----|
| `turn` | Power on/off | `on`, `off` | ✓ | ✓ |
| `brightness` | Set brightness | 0-100% | ✓ | ✓ |
| `color` | Set RGB color | 0-255 per channel | ✓ | ✓ |
| `temp` | Set color temperature | 2000-9000K | ✓ | ✓ |
| `scene` | Apply light/DIY scenes | per device | ✓ | ✓ |

## Development

```bash
cargo build                        # Build both crates (workspace)
cargo test                         # Run all tests (incl. govee-api2 suite)
cargo fmt && cargo clippy -- -D warnings
```

## Project Structure

```
govee-tui/
├── src/
│   ├── main.rs          # Entry point + CLI args
│   ├── config.rs        # Configuration management
│   ├── cache.rs         # Device-list cache (instant startup)
│   ├── api/             # Thin wrapper over govee-api2
│   └── ui/              # TUI: app state, event loop, widgets
├── govee-api2/          # Bundled Govee v2 platform-API client (tested against a mock server)
├── .github/workflows/   # CI, release, and security-audit pipelines
└── Cargo.toml
```

## CI/CD

- **Format / Lint / Test**: `rustfmt`, `clippy -D warnings`, full test suite on Linux + macOS
- **Release**: tag `v*.*.*` -> binaries for Linux (musl) and macOS (x86_64 + aarch64)
- **Security audit**: weekly `cargo audit`

## Roadmap

- **LAN API support** - local UDP control: instant, offline, no API key, no rate limits. The single biggest upgrade on this list.
- **Device search / filter** in the TUI
- **Multi-device selection** and batched commands (turn a whole room off)
- **Segmented color** - the API client already supports it; needs UI

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Issues and pull requests welcome. Before opening a PR:

1. Fork the repository and create a feature branch
2. Make sure `cargo fmt` and `cargo clippy -- -D warnings` pass
3. Add tests for new behavior
4. Open a pull request against `main`

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Talks to Govee devices via the bundled [`govee-api2`](govee-api2/) crate in this repo - a client for Govee's v2 platform API with scenes, segments, retry, and rate-limit handling
