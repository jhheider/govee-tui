# Govee TUI - Project Proposal

## Overview
A clean, modular Rust TUI application for controlling Govee smart home devices with beautiful colorful UI, efficient architecture, and robust CI/CD.

## Architecture

### Single Crate with Modules
```
govee-tui/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point + CLI args (~90 lines)
│   ├── api/
│   │   ├── mod.rs       # Govee API client (~80 lines)
│   │   ├── models.rs    # Device models (~80 lines)
│   │   └── commands.rs  # Control commands (~90 lines)
│   ├── db/
│   │   ├── mod.rs       # SQLite connection (~70 lines)
│   │   ├── schema.rs    # Database schema (~60 lines)
│   │   └── cache.rs     # Device/state cache (~80 lines)
│   ├── ui/
│   │   ├── mod.rs       # App state & event loop (~90 lines)
│   │   ├── theme.rs     # Colors & emojis (~60 lines)
│   │   └── widgets/
│   │       ├── mod.rs           # Widget exports (~20 lines)
│   │       ├── device_list.rs   # Device list view (~80 lines)
│   │       ├── detail_view.rs   # Device details (~90 lines)
│   │       ├── color_picker.rs  # RGB color picker (~90 lines)
│   │       └── brightness.rs    # Brightness slider (~60 lines)
│   └── config.rs        # Configuration management (~80 lines)
```
**Total: ~1,100 lines across 16 files**

## Govee API Quick Reference

The `govee-api` crate provides thin wrappers around Govee's REST API:

```rust
// List all devices
client.get_devices().await?

// Control device (generic method)
client.control_device(device_id, Command {
    name: "turn",        // or "brightness", "color", "colorTem"
    value: Value::On     // Command-specific value
}).await?
```

**Command Types:**
- `turn`: `Value::On` / `Value::Off`
- `brightness`: `Value::Int(0..=100)`
- `color`: `Value::Color { r, g, b }` (each 0-255)
- `colorTem`: `Value::Int(2000..=9000)` (Kelvin)

## Core Dependencies

### Runtime
- `govee-api` (latest) - Govee device control
- `ratatui` (latest) - TUI framework (chosen over slint for terminal focus)
- `crossterm` - Terminal manipulation
- `clap` (4.x) - CLI argument parsing with derive
- `rusqlite` (latest) - SQLite with bundled static compilation
- `tokio` (1.x) - Async runtime (required by govee-api)
- `serde` + `serde_json` - Serialization
- `anyhow` - Error handling
- `tracing` + `tracing-subscriber` - Structured logging
- `chrono` - Timestamps
- `unicode-width` - Text layout

### Development
- `cargo-watch` - Hot reload during dev
- `cargo-nextest` - Parallel testing
- `cargo-deny` - License/security checks

## Features Breakdown

### 1. Device Management (govee-core)
```rust
// Core operations
- List all devices (with caching)
- Get device details (state, capabilities, model)
- Search/filter devices by name, type, location
- Group management (rooms, zones)
```

### 2. Device Control (API Commands)
```rust
// Govee API supports these control commands:

// "turn" - Power control
Turn::On | Turn::Off

// "brightness" - Brightness control
Brightness(0..=100)  // Percentage (0-100)

// "color" - RGB color control
Color { r: 0..=255, g: 0..=255, b: 0..=255 }

// "colorTem" - Color temperature control
ColorTemp(2000..=9000)  // Kelvin (warm to cool)

// Fine-grained UI controls:
- Arrow keys: ±5% brightness, ±100K temperature
- Shift+Arrow: ±1% brightness, ±10K temperature
- Mouse/trackpad: Precise RGB color picker
- Number keys: Quick brightness (1=10%, 2=20%, etc.)
- Color presets: Common colors + favorites
```

### 3. Data Persistence (govee-db)
```sql
-- Schema design
devices (id, name, model, mac, capabilities, last_seen)
device_state (device_id, state_json, updated_at)
command_history (device_id, command, timestamp, success)
preferences (key, value) -- UI settings, favorites
cache (key, value, expires_at) -- API response cache
```

### 4. TUI Interface (govee-ui)
```
┌─ Govee Controller ─────────────────────────────────┐
│ 🏠 Devices (5) │ 📊 Stats │ ⚙️  Settings │ ❓ Help  │
├────────────────────────────────────────────────────┤
│ 📱 Device List        │ 🔍 Device Details          │
│ ┌──────────────────┐  │ ┌─────────────────────────┐│
│ │ 💡 Living Room   │◀─┼─│ 💡 Living Room LED      ││
│ │ ✅ ON  100% 🌈  │  │ │ Model: H6163            ││
│ │                  │  │ │ MAC: XX:XX:XX:XX:XX:XX  ││
│ │ 💡 Bedroom       │  │ │                         ││
│ │ ⭕ OFF           │  │ │ Power: ✅ ON            ││
│ │                  │  │ │ Brightness: ████████░░  ││
│ │ 🌡️  Desk Strip   │  │ │ Color: 🔴 Red (RGB)    ││
│ │ ✅ ON   75% ☀️   │  │ │ Temp: 4000K ☀️          ││
│ └──────────────────┘  │ │                         ││
│ [F1] Refresh          │ │ [↑↓] Brightness         ││
│ [Enter] Select        │ │ [C] Color [T] Temp      ││
│ [Space] Toggle        │ │ [S] Scenes [R] Refresh  ││
│ [Q] Quit              │ └─────────────────────────┘│
├────────────────────────────────────────────────────┤
│ 📡 API: Connected │ 💾 DB: 15 devices │ 🕐 16:20  │
└────────────────────────────────────────────────────┘
```

**UI Features:**
- Tab navigation between views
- Vim-style keybindings (hjkl + arrow keys)
- Real-time device state updates
- Smooth color picker with preview
- Command palette (Ctrl+P)
- Search/filter (Ctrl+F)
- Emoji indicators for status
- Color gradients for brightness/temp
- Toast notifications for actions

## CI/CD Pipeline (GitHub Actions)

### Workflow: `ci.yml`
```yaml
Strategy:
- Matrix: [ubuntu-latest, macos-latest] only (no Windows, single Rust stable)
- Parallelism: All jobs run concurrently
- Efficient caching to minimize build times

Jobs:
1. Format Check (rustfmt) - ~30s
2. Lint (clippy --deny warnings) - ~2min
3. Test (cargo nextest) - ~1min
4. Build (debug build for validation) - ~2min

Optimizations:
- Swatinem/rust-cache@v2 - Smart incremental caching
- Sparse registry (CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse)
- Zero apt dependencies - pure Cargo/Rustup
- cargo-binstall for tools (faster than compilation)

Target: <5 minute total CI time
```

### Workflow: `release.yml`
```yaml
Trigger: Git tags (v*.*.*)

Targets (using cargo-zigbuild for easy cross-compilation):
- x86_64-unknown-linux-musl (static, no glibc dependency)
- x86_64-apple-darwin (Intel Macs)
- aarch64-apple-darwin (M1/M2/M3 Macs)

Steps:
1. Cross-compile release builds
2. Strip binaries, compress with upx
3. Generate SHA256 checksums
4. Create GitHub Release with binaries
5. Optional: cargo publish to crates.io
```

### Workflow: `audit.yml` (Weekly)
```yaml
Trigger: Cron (Sundays at 00:00 UTC)

Purpose:
- cargo-audit for security vulnerabilities
- cargo-outdated for dependency updates
- Create issue if action needed (not auto-PR to avoid noise)
```

## Development Guidelines

### Code Quality Standards
```toml
# .cargo/config.toml
[build]
rustflags = ["-D", "warnings"]

[target.x86_64-unknown-linux-musl]
linker = "rust-lld"  # Fast linking
```

**Mandatory Checks:**
- `cargo fmt --check` must pass
- `cargo clippy -- -D warnings` must pass
- Max file length: ~100 lines (strict guideline)
- Test coverage: >80% for core business logic
- No `unwrap()` in production code (use `?` or `expect()`)
- All public APIs documented

### Project Principles
1. **DRY**: Extract common patterns, avoid duplication
2. **Modularity**: Clear module boundaries, single responsibility
3. **Pragmatic**: Don't over-engineer, modules > workspace crates
4. **Testability**: Mock external APIs, test business logic
5. **Performance**: Cache responses, efficient async operations
6. **UX**: Responsive UI, clear feedback, graceful errors

## Configuration

### User Config File (`~/.config/govee-tui/config.toml`)
```toml
[api]
key = "your-govee-api-key"
timeout_ms = 5000
retry_attempts = 3

[ui]
theme = "dark"  # dark, light, auto
emoji = true
refresh_interval_ms = 5000

[database]
path = "~/.local/share/govee-tui/devices.db"
cache_ttl_seconds = 300

[keybindings]
quit = "q"
refresh = "r"
# ... customizable
```

## Implementation Phases

### Phase 1: Foundation (Days 1-2)
- [ ] Cargo.toml with dependencies
- [ ] CI/CD pipeline (fmt, clippy, test, build)
- [ ] Module structure (api/, db/, ui/, config.rs)
- [ ] Basic CLI argument parsing (clap)
- [ ] API client wrapper with device listing

### Phase 2: Core Logic (Days 3-4)
- [ ] All 4 control commands (turn, brightness, color, colorTem)
- [ ] SQLite schema + device/state caching
- [ ] Error handling & logging (tracing)
- [ ] Unit tests for control logic

### Phase 3: TUI (Days 5-7)
- [ ] Basic ratatui app + event loop
- [ ] Device list widget with status emojis
- [ ] Device detail view with live updates
- [ ] RGB color picker + temperature slider
- [ ] Brightness control (arrow keys, number shortcuts)
- [ ] Theme & emoji styling

### Phase 4: Polish (Days 8-9)
- [ ] Fine-grained controls (Shift+arrows for precision)
- [ ] Search/filter UI (Ctrl+F)
- [ ] Configuration file support
- [ ] Error handling & user feedback
- [ ] README with screenshots/demo

### Phase 5: Release (Day 10)
- [ ] Release workflow (cross-compilation)
- [ ] Binary artifacts for Linux/macOS
- [ ] Installation guide
- [ ] Tag v0.1.0

## Success Metrics
- ✅ Zero clippy warnings (always)
- ✅ 100% fmt compliance (always)
- ✅ All files ~100 lines or less
- ✅ CI completes in <5 minutes
- ✅ Linux + macOS builds passing
- ✅ Single binary, statically linked
- ✅ <2s cold start time
- ✅ <50ms UI response time

## Nice-to-Have Features (Future)
- Multi-device batch control
- Schedules/automation (cron-style)
- Device state monitoring dashboard
- Export/import configurations
- Plugin system for custom commands
- Web dashboard (actix-web + htmx)

---

**Ready to implement?** This proposal prioritizes clean architecture, efficient CI/CD, and maintainable code. Each module is small, focused, and testable.
