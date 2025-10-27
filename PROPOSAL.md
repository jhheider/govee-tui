# Govee TUI - Project Proposal

## Overview
A clean, modular Rust TUI application for controlling Govee smart home devices with beautiful colorful UI, efficient architecture, and robust CI/CD.

## Architecture

### Workspace Structure (Multi-Crate)
```
govee-tui/
├── Cargo.toml (workspace)
├── crates/
│   ├── govee-core/      # Business logic, API wrapper, ~400 lines total
│   │   ├── api.rs       # Govee API client (~80 lines)
│   │   ├── models.rs    # Device models (~80 lines)
│   │   ├── commands.rs  # Device commands (~80 lines)
│   │   └── lib.rs       # Module exports (~30 lines)
│   ├── govee-db/        # SQLite persistence, ~300 lines total
│   │   ├── schema.rs    # Database schema (~60 lines)
│   │   ├── store.rs     # Device storage (~80 lines)
│   │   ├── cache.rs     # API cache layer (~80 lines)
│   │   └── lib.rs       # Module exports (~30 lines)
│   ├── govee-ui/        # Ratatui TUI components, ~600 lines total
│   │   ├── app.rs       # App state (~90 lines)
│   │   ├── widgets/     # Custom widgets
│   │   │   ├── device_list.rs (~80 lines)
│   │   │   ├── device_detail.rs (~80 lines)
│   │   │   ├── color_picker.rs (~90 lines)
│   │   │   └── mod.rs (~20 lines)
│   │   ├── theme.rs     # Colors & emojis (~80 lines)
│   │   ├── events.rs    # Input handling (~80 lines)
│   │   └── lib.rs       # Module exports (~30 lines)
│   └── govee-cli/       # Main binary, ~200 lines total
│       ├── main.rs      # Entry point (~80 lines)
│       ├── args.rs      # Clap definitions (~60 lines)
│       └── runner.rs    # Execution logic (~60 lines)
```

## Core Dependencies

### Runtime
- `govee-api` (0.7+) - Govee device control
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

### 2. Device Control (govee-core)
```rust
// Control commands
- Power on/off
- Brightness adjustment (0-100%)
- Color setting (RGB, presets)
- Color temperature (kelvin)
- Scene activation
- Command history
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
- Matrix build: [ubuntu-latest, macos-latest, windows-latest]
- Rust versions: [stable, beta]
- Parallelism: All jobs run concurrently
- Caching: Cargo registry, git deps, target/ (sccache for faster compilation)

Jobs:
1. Format Check (rustfmt)
2. Lint (clippy --deny warnings)
3. Test (cargo nextest - 3x faster than cargo test)
4. Security Audit (cargo-deny, cargo-audit)
5. Build (release builds for all platforms)
6. Coverage (cargo-tarpaulin → Codecov)

Optimizations:
- Rust cache action (Swatinem/rust-cache@v2)
- Sparse registry protocol (CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse)
- No apt dependencies (everything via cargo/rustup)
- Incremental compilation in CI
```

### Workflow: `release.yml`
```yaml
Trigger: Git tags (v*.*.*)

Steps:
1. Cross-compile for multiple targets:
   - x86_64-unknown-linux-gnu (static musl)
   - x86_64-apple-darwin
   - aarch64-apple-darwin (M1/M2 Macs)
   - x86_64-pc-windows-gnu
2. Generate checksums (SHA256)
3. Create GitHub Release
4. Upload binaries as artifacts
5. Optional: Publish to crates.io
```

### Workflow: `deps.yml` (Weekly)
```yaml
Trigger: Cron schedule

Purpose:
- cargo update --dry-run
- Check for outdated dependencies
- Security advisories
- Create automated PR if updates available
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
1. **DRY**: Extract common patterns into shared modules
2. **Modularity**: Each crate has single responsibility
3. **Testability**: Mock external APIs, test business logic
4. **Performance**: Cache API responses, async operations
5. **UX**: Responsive UI, clear feedback, graceful errors

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
- [ ] Workspace setup with all crates
- [ ] CI/CD pipeline (fmt, clippy, test)
- [ ] Basic CLI argument parsing
- [ ] govee-core: API client wrapper
- [ ] govee-db: SQLite schema + migrations

### Phase 2: Core Logic (Days 3-4)
- [ ] Device listing & filtering
- [ ] Device control commands
- [ ] Caching layer
- [ ] Error handling & logging
- [ ] Unit tests for business logic

### Phase 3: TUI (Days 5-7)
- [ ] Basic ratatui app structure
- [ ] Device list widget
- [ ] Device detail view
- [ ] Color picker widget
- [ ] Theme & styling
- [ ] Keybinding system

### Phase 4: Polish (Days 8-9)
- [ ] Command history
- [ ] Search/filter UI
- [ ] Configuration management
- [ ] Integration tests
- [ ] Documentation
- [ ] Release builds

### Phase 5: Deployment (Day 10)
- [ ] Release automation
- [ ] Installation instructions
- [ ] Demo GIF/screenshots
- [ ] Crates.io publication

## Success Metrics
- ✅ Zero clippy warnings
- ✅ 100% fmt compliance
- ✅ <3s cold start time
- ✅ <100ms UI response time
- ✅ All files under 100 lines
- ✅ CI runs in <5 minutes
- ✅ Cross-platform builds successful
- ✅ Zero unsafe code (unless absolutely necessary)

## Nice-to-Have Features (Future)
- Multi-device batch control
- Schedules/automation (cron-style)
- Device state monitoring dashboard
- Export/import configurations
- Plugin system for custom commands
- Web dashboard (actix-web + htmx)

---

**Ready to implement?** This proposal prioritizes clean architecture, efficient CI/CD, and maintainable code. Each module is small, focused, and testable.
