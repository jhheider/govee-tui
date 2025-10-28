# Govee API Test Scripts

Direct `curl` + `jq` test scripts for debugging the Govee API without the TUI.

## Requirements

```bash
# macOS
brew install curl jq

# Ubuntu/Debian
sudo apt-get install curl jq

# Fedora
sudo dnf install curl jq
```

## Setup

Set your API key using one of these methods:

```bash
# Option 1: Environment variable
export GOVEE_API_KEY="your-api-key-here"

# Option 2: Already configured in govee-tui
# Scripts will auto-load from ~/.config/govee-tui/config.toml

# Option 3: Pass as argument to each script
./scripts/get-devices.sh "your-api-key-here"
```

## Quick Start

### Interactive Menu
```bash
./scripts/test-api.sh
```

This launches an interactive menu with all tests.

### Individual Scripts

#### 1. List All Devices
```bash
./scripts/get-devices.sh

# With verbose API key
./scripts/get-devices.sh "your-api-key"
```

Output:
- Device count
- Table of all devices (Model, ID, Name)
- Capabilities summary
- Saves raw JSON to `/tmp/govee-devices.json`

#### 2. Get Device State
```bash
./scripts/get-device-state.sh <DEVICE_ID> <MODEL>

# Example
./scripts/get-device-state.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159"
```

Output:
- Current power state (ON/OFF)
- Brightness level
- RGB color
- Color temperature

#### 3. Control Device
```bash
./scripts/control-device.sh <DEVICE_ID> <MODEL> <COMMAND>

# Turn on/off
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" on
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" off

# Set brightness (0-100)
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" brightness:75

# Set RGB color (0-255 each)
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" color:255,128,0

# Set color temperature (2000-9000K)
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" temp:4500
```

## Debugging Missing Devices

If you're seeing fewer devices than expected (e.g., 8 instead of 16):

### Step 1: Check API Response
```bash
./scripts/get-devices.sh

# Look for:
# - Device count in output
# - Check /tmp/govee-devices.json for full response
```

### Step 2: Compare Endpoints
```bash
# Interactive menu option 5, or manually:
API_KEY="your-key"

# New endpoint (used by govee-tui)
curl -s "https://openapi.api.govee.com/router/api/v1/user/devices" \
  -H "Govee-API-Key: $API_KEY" | jq '.data | length'

# Legacy endpoint (possibly used by govee-api crate)
curl -s "https://openapi.api.govee.com/v1/devices" \
  -H "Govee-API-Key: $API_KEY" | jq '.data.devices | length'
```

### Step 3: Check Response Structure
```bash
# View full structure
cat /tmp/govee-devices.json | jq .

# Count devices
jq '.data | length' /tmp/govee-devices.json

# List all device IDs
jq -r '.data[].device' /tmp/govee-devices.json
```

## API Endpoints

### New API (Router-based)
- **List Devices**: `GET /router/api/v1/user/devices`
- **Device State**: `GET /router/api/v1/device/state?device=xxx&model=xxx`
- **Control**: `POST /router/api/v1/device/control`

### Legacy API
- **List Devices**: `GET /v1/devices`
- **Device State**: `GET /v1/devices/state?device=xxx&model=xxx`
- **Control**: `PUT /v1/devices/control`

**Note:** The `govee-api` crate v1.3 may use legacy endpoints, which could explain device count differences.

## Troubleshooting

### Authentication Failed (401)
- Check API key is correct
- Verify key is active in Govee Home app

### Rate Limit (429)
- Govee allows 10,000 requests per account per day
- Wait and try again later

### Device Not Found (404)
- Verify device ID and model are correct
- Run `get-devices.sh` to see available devices
- Some devices may not support state retrieval

### No Devices Returned
- Check API key permissions
- Ensure devices are added in Govee Home app
- Try comparing new vs legacy endpoints

## Tips

1. **Cache device list**: `get-devices.sh` saves to `/tmp/govee-devices.json`
2. **Use interactive menu**: `test-api.sh` is easiest for exploration
3. **Pipe to jq**: All scripts output valid JSON for further processing
4. **Check both endpoints**: Some devices may only show on one endpoint

## Example Workflow

```bash
# 1. List all devices and save to cache
./scripts/get-devices.sh

# 2. Get device info (interactive selection)
./scripts/test-api.sh
# Choose option 4 for quick device test

# 3. Control a specific device
./scripts/control-device.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159" on

# 4. Verify state changed
./scripts/get-device-state.sh "AA:BB:CC:DD:EE:FF:11:22" "H6159"
```

## API Documentation

Official Govee API docs: https://developer.govee.com/reference

## Contributing

These scripts are designed to be:
- **Simple**: Pure bash + curl + jq
- **Portable**: Work on any Unix-like system
- **Debuggable**: Show all requests and responses
- **Extensible**: Easy to modify for new endpoints
