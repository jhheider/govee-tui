#!/bin/bash
# Master test script for Govee API
# Interactive menu for testing API endpoints

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check dependencies
check_deps() {
    local missing=()
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v jq >/dev/null 2>&1 || missing+=("jq")

    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}❌ Missing dependencies: ${missing[*]}${NC}"
        echo ""
        echo "Install with:"
        echo "  macOS: brew install ${missing[*]}"
        echo "  Ubuntu/Debian: sudo apt-get install ${missing[*]}"
        echo "  Fedora: sudo dnf install ${missing[*]}"
        exit 1
    fi
}

# Get API key
get_api_key() {
    if [ -n "$GOVEE_API_KEY" ]; then
        echo "$GOVEE_API_KEY"
        return
    fi

    CONFIG_FILE="$HOME/.config/govee-tui/config.toml"
    if [ -f "$CONFIG_FILE" ]; then
        grep '^key = ' "$CONFIG_FILE" | cut -d'"' -f2
    fi
}

# Show banner
show_banner() {
    clear
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BLUE}🧪 Govee API Test Suite${NC}                                 ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}     Direct curl + jq testing without TUI                  ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    API_KEY=$(get_api_key)
    if [ -n "$API_KEY" ] && [ "$API_KEY" != "YOUR_API_KEY_HERE" ]; then
        echo -e "${GREEN}✓ API Key configured${NC} (${API_KEY:0:8}...${API_KEY: -4})"
    else
        echo -e "${YELLOW}⚠ No API key found${NC}"
        echo "  Set via: export GOVEE_API_KEY='your-key'"
        echo "  Or in: ~/.config/govee-tui/config.toml"
    fi
    echo ""
}

# Main menu
show_menu() {
    echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Select a test:${NC}"
    echo ""
    echo "  ${GREEN}1${NC}) List all devices          📱 GET /router/api/v1/user/devices"
    echo "  ${GREEN}2${NC}) Get device state          📊 GET /router/api/v1/device/state"
    echo "  ${GREEN}3${NC}) Control device            🎮 POST /router/api/v1/device/control"
    echo ""
    echo "  ${GREEN}4${NC}) Test specific device      🔍 Quick device lookup + test"
    echo "  ${GREEN}5${NC}) Compare endpoints         🔄 New vs Legacy API"
    echo ""
    echo "  ${GREEN}q${NC}) Quit"
    echo ""
    echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
    echo -n "Enter choice: "
}

# Test 1: List devices
test_list_devices() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Test 1: List All Devices${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    bash "$SCRIPT_DIR/get-devices.sh"

    echo ""
    read -p "Press Enter to continue..."
}

# Test 2: Get device state
test_device_state() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Test 2: Get Device State${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    # Check if we have cached devices
    if [ -f /tmp/govee-devices.json ]; then
        echo -e "${CYAN}Available devices (from cache):${NC}"
        jq -r '.data[] | "\(.device) - \(.deviceName) (\(.sku))"' /tmp/govee-devices.json | nl
        echo ""
    else
        echo -e "${YELLOW}Tip: Run test 1 first to see available devices${NC}"
        echo ""
    fi

    read -p "Device ID: " device_id
    read -p "Model: " model

    if [ -n "$device_id" ] && [ -n "$model" ]; then
        bash "$SCRIPT_DIR/get-device-state.sh" "$device_id" "$model"
    else
        echo -e "${RED}Cancelled${NC}"
    fi

    echo ""
    read -p "Press Enter to continue..."
}

# Test 3: Control device
test_control_device() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Test 3: Control Device${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    # Check if we have cached devices
    if [ -f /tmp/govee-devices.json ]; then
        echo -e "${CYAN}Available devices (from cache):${NC}"
        jq -r '.data[] | "\(.device) - \(.deviceName) (\(.sku))"' /tmp/govee-devices.json | nl
        echo ""
    fi

    read -p "Device ID: " device_id
    read -p "Model: " model

    if [ -z "$device_id" ] || [ -z "$model" ]; then
        echo -e "${RED}Cancelled${NC}"
        read -p "Press Enter to continue..."
        return
    fi

    echo ""
    echo "Commands:"
    echo "  on, off"
    echo "  brightness:50"
    echo "  color:255,128,0"
    echo "  temp:4500"
    echo ""
    read -p "Command: " command

    if [ -n "$command" ]; then
        bash "$SCRIPT_DIR/control-device.sh" "$device_id" "$model" "$command"
    else
        echo -e "${RED}Cancelled${NC}"
    fi

    echo ""
    read -p "Press Enter to continue..."
}

# Test 4: Quick device test
test_quick_device() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Test 4: Quick Device Test${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    echo "First, let's get your devices..."
    bash "$SCRIPT_DIR/get-devices.sh" | head -20

    if [ ! -f /tmp/govee-devices.json ]; then
        echo -e "${RED}Failed to fetch devices${NC}"
        read -p "Press Enter to continue..."
        return
    fi

    echo ""
    echo -e "${CYAN}Select a device by number:${NC}"
    jq -r '.data[] | "\(.device) - \(.deviceName) (\(.sku))"' /tmp/govee-devices.json | nl
    echo ""
    read -p "Device #: " device_num

    if ! [[ "$device_num" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}Invalid selection${NC}"
        read -p "Press Enter to continue..."
        return
    fi

    device_id=$(jq -r ".data[$((device_num - 1))].device" /tmp/govee-devices.json)
    model=$(jq -r ".data[$((device_num - 1))].sku" /tmp/govee-devices.json)
    name=$(jq -r ".data[$((device_num - 1))].deviceName" /tmp/govee-devices.json)

    echo ""
    echo -e "${GREEN}Testing: ${name}${NC}"
    echo -e "${YELLOW}ID:${NC} ${device_id}"
    echo -e "${YELLOW}Model:${NC} ${model}"
    echo ""

    echo -e "${CYAN}Fetching current state...${NC}"
    bash "$SCRIPT_DIR/get-device-state.sh" "$device_id" "$model"

    echo ""
    read -p "Send a test command? (y/N): " confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        echo ""
        read -p "Command (on/off/brightness:50): " command
        if [ -n "$command" ]; then
            bash "$SCRIPT_DIR/control-device.sh" "$device_id" "$model" "$command"
        fi
    fi

    echo ""
    read -p "Press Enter to continue..."
}

# Test 5: Compare endpoints
test_compare_endpoints() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Test 5: Compare New vs Legacy Endpoints${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    API_KEY=$(get_api_key)
    BASE_URL="https://openapi.api.govee.com"

    echo -e "${CYAN}Testing NEW endpoint: /router/api/v1/user/devices${NC}"
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        "${BASE_URL}/router/api/v1/user/devices" \
        -H "Govee-API-Key: ${API_KEY}" \
        -H "Content-Type: application/json")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    echo "  Status: ${HTTP_CODE}"
    if [ "$HTTP_CODE" = "200" ]; then
        DEVICE_COUNT=$(echo "$BODY" | jq -r '.data | length')
        echo "  Devices: ${DEVICE_COUNT}"
    else
        echo "  Error: $(echo "$BODY" | jq -r '.message // .error // "Unknown"')"
    fi

    echo ""
    echo -e "${CYAN}Testing LEGACY endpoint: /v1/devices${NC}"
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        "${BASE_URL}/v1/devices" \
        -H "Govee-API-Key: ${API_KEY}" \
        -H "Content-Type: application/json")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    echo "  Status: ${HTTP_CODE}"
    if [ "$HTTP_CODE" = "200" ]; then
        DEVICE_COUNT=$(echo "$BODY" | jq -r '.data.devices | length')
        echo "  Devices: ${DEVICE_COUNT}"
    else
        echo "  Error: $(echo "$BODY" | jq -r '.message // .error // "Unknown"')"
    fi

    echo ""
    echo -e "${YELLOW}Note: govee-api crate v1.3 likely uses legacy /v1/devices endpoint${NC}"
    echo -e "${YELLOW}This may explain device count discrepancies${NC}"

    echo ""
    read -p "Press Enter to continue..."
}

# Main loop
main() {
    check_deps

    while true; do
        show_banner
        show_menu
        read -r choice

        case "$choice" in
            1) test_list_devices ;;
            2) test_device_state ;;
            3) test_control_device ;;
            4) test_quick_device ;;
            5) test_compare_endpoints ;;
            q|Q) echo -e "\n${GREEN}Goodbye!${NC}\n"; exit 0 ;;
            *) echo -e "${RED}Invalid choice${NC}"; sleep 1 ;;
        esac
    done
}

main
