# CLAUDE.md

This file provides guidance when working with code in this repository.

## Build and Test Commands

```bash
# Run all checks (formatting, linting, testing)
just check

# Individual commands
just fmt          # Check formatting
just clippy       # Run linter
just test         # Run all tests

# Run a single test
cargo test --package govee-api2 test_name
cargo test --package govee-tui test_name

# Build release binary
just build        # or: cargo build --release

# Build just the library (faster, no TUI deps)
cargo build --package govee-api2
```

## Architecture

This is a Rust workspace with two crates:

### govee-api2 (library crate)

A Rust client for Govee's v2 router-based platform API at `https://openapi.api.govee.com`.

**Key modules:**
- `client.rs` - `GoveeClient` with retry/backoff, rate-limit handling, all device operations
- `types.rs` - Data types (Device, Capability, Scene, DeviceState, Color)
- `error.rs` - Typed error enum (Request, Api, InvalidApiKey, RateLimited, Server, etc.)

**Testing:** Uses `wiremock` for API client tests (in client.rs). Unit tests for types in types.rs.

### govee-tui (binary crate)

A TUI application built with `ratatui` and `crossterm` for controlling Govee devices interactively.

**Key modules:**
- `main.rs` - CLI entry point with clap subcommands (TUI, Devices, Status, Scenes, Control)
- `api/` - Client wrapper (`api::Client`) and command types
- `config.rs` - TOML config file loading with env var override
- `cache.rs` - Atomic write-then-rename device list cache
- `ui/` - Ratatui-based terminal interface with multiple widgets

## Test Organization

- `govee-api2/src/types.rs` - Device capability detection, state parsing, scene extraction, color packing
- `govee-api2/src/error.rs` - Error display formatting, Send+Sync impls
- `govee-api2/src/client.rs` - Client construction, request ID generation, retry-after header parsing
- `govee-tui/src/main.rs` - CLI helpers (find_device, truncate)
- `govee-tui/src/config.rs` - Default values, env var override, serde round-trip
- `govee-tui/src/cache.rs` - Atomic write-then-rename cache file operations

## Code Style

- All public API items documented
- No `unwrap()` in production code (use `?` or expect with context)
- Tracing for observability (info/debug/warn), not println
- `cargo clippy -- -D warnings` must pass
