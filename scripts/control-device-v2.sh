#!/bin/bash
# Test script to control a Govee device using router API v2
# Usage: ./scripts/control-device-v2.sh <DEVICE_ID> <MODEL> <COMMAND> [API_KEY]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
DEVICE_ID="$1"
MODEL="$2"
COMMAND="$3"
API_KEY="${4:-${GOVEE_API_KEY}}"

if [ -z "$DEVICE_ID" ] || [ -z "$MODEL" ] || [ -z "$COMMAND" ]; then
    echo -e "${RED}❌ Error: Missing arguments${NC}"
    echo ""
    echo "Usage:"
    echo "  $0 <DEVICE_ID> <MODEL> <COMMAND> [API_KEY]"
    echo ""
    echo "Commands:"
    echo "  on                   - Turn device on"
    echo "  off                  - Turn device off"
    echo "  brightness:<0-100>   - Set brightness"
    echo "  color:<r>,<g>,<b>    - Set RGB color (0-255 each)"
    echo "  temp:<2000-9000>     - Set color temperature in Kelvin"
    exit 1
fi

# Get API key from config if not provided
if [ -z "$API_KEY" ]; then
    CONFIG_FILE="$HOME/.config/govee-tui/config.toml"
    if [ -f "$CONFIG_FILE" ]; then
        API_KEY=$(grep '^key = ' "$CONFIG_FILE" | cut -d'"' -f2)
    fi
fi

if [ -z "$API_KEY" ] || [ "$API_KEY" = "YOUR_API_KEY_HERE" ]; then
    echo -e "${RED}❌ Error: No API key provided${NC}"
    exit 1
fi

# Check dependencies
command -v curl >/dev/null 2>&1 || { echo -e "${RED}❌ curl is required${NC}"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo -e "${RED}❌ jq is required${NC}"; exit 1; }

echo -e "${BLUE}🎮 Sending control command (Router API v2)...${NC}"
echo ""

# Generate request ID
REQUEST_ID="bash-$(date +%s%3N)"

# Parse command and build payload
case "$COMMAND" in
    on)
        PAYLOAD=$(jq -n --arg requestId "$REQUEST_ID" --arg device "$DEVICE_ID" --arg sku "$MODEL" \
            '{requestId: $requestId, payload: {sku: $sku, device: $device, capability: {type: "devices.capabilities.on_off", instance: "powerSwitch", value: 1}}}')
        echo -e "${YELLOW}Command:${NC} Turn ON"
        ;;
    off)
        PAYLOAD=$(jq -n --arg requestId "$REQUEST_ID" --arg device "$DEVICE_ID" --arg sku "$MODEL" \
            '{requestId: $requestId, payload: {sku: $sku, device: $device, capability: {type: "devices.capabilities.on_off", instance: "powerSwitch", value: 0}}}')
        echo -e "${YELLOW}Command:${NC} Turn OFF"
        ;;
    brightness:*)
        VALUE="${COMMAND#brightness:}"
        if ! [[ "$VALUE" =~ ^[0-9]+$ ]] || [ "$VALUE" -lt 0 ] || [ "$VALUE" -gt 100 ]; then
            echo -e "${RED}❌ Invalid brightness value (must be 0-100)${NC}"
            exit 1
        fi
        PAYLOAD=$(jq -n --arg requestId "$REQUEST_ID" --arg device "$DEVICE_ID" --arg sku "$MODEL" --argjson value "$VALUE" \
            '{requestId: $requestId, payload: {sku: $sku, device: $device, capability: {type: "devices.capabilities.range", instance: "brightness", value: $value}}}')
        echo -e "${YELLOW}Command:${NC} Set brightness to ${VALUE}%"
        ;;
    color:*)
        RGB="${COMMAND#color:}"
        IFS=',' read -r R G B <<< "$RGB"
        if ! [[ "$R" =~ ^[0-9]+$ ]] || ! [[ "$G" =~ ^[0-9]+$ ]] || ! [[ "$B" =~ ^[0-9]+$ ]]; then
            echo -e "${RED}❌ Invalid color format (use r,g,b with 0-255 each)${NC}"
            exit 1
        fi
        # Pack RGB into single integer: (r << 16) | (g << 8) | b
        PACKED_RGB=$(( (R << 16) | (G << 8) | B ))
        PAYLOAD=$(jq -n --arg requestId "$REQUEST_ID" --arg device "$DEVICE_ID" --arg sku "$MODEL" --argjson value "$PACKED_RGB" \
            '{requestId: $requestId, payload: {sku: $sku, device: $device, capability: {type: "devices.capabilities.color_setting", instance: "colorRgb", value: $value}}}')
        echo -e "${YELLOW}Command:${NC} Set color to RGB(${R}, ${G}, ${B}) = packed ${PACKED_RGB}"
        ;;
    temp:*)
        VALUE="${COMMAND#temp:}"
        if ! [[ "$VALUE" =~ ^[0-9]+$ ]] || [ "$VALUE" -lt 2000 ] || [ "$VALUE" -gt 9000 ]; then
            echo -e "${RED}❌ Invalid temperature value (must be 2000-9000K)${NC}"
            exit 1
        fi
        PAYLOAD=$(jq -n --arg requestId "$REQUEST_ID" --arg device "$DEVICE_ID" --arg sku "$MODEL" --argjson value "$VALUE" \
            '{requestId: $requestId, payload: {sku: $sku, device: $device, capability: {type: "devices.capabilities.color_setting", instance: "colorTemperatureK", value: $value}}}')
        echo -e "${YELLOW}Command:${NC} Set color temperature to ${VALUE}K"
        ;;
    *)
        echo -e "${RED}❌ Unknown command: ${COMMAND}${NC}"
        exit 1
        ;;
esac

echo -e "${YELLOW}Device ID:${NC} ${DEVICE_ID}"
echo -e "${YELLOW}Model:${NC} ${MODEL}"
echo ""

# Show payload
echo -e "${YELLOW}Request Payload:${NC}"
echo "$PAYLOAD" | jq .
echo ""

# API endpoint
BASE_URL="https://openapi.api.govee.com"
ENDPOINT="/router/api/v1/device/control"

# Make request
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    "${BASE_URL}${ENDPOINT}" \
    -H "Govee-API-Key: ${API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo -e "${YELLOW}HTTP Status:${NC} ${HTTP_CODE}"
echo ""

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Command executed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}Response:${NC}"
    echo "$BODY" | jq .

elif [ "$HTTP_CODE" = "401" ]; then
    echo -e "${RED}❌ Authentication failed (401)${NC}"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

elif [ "$HTTP_CODE" = "404" ]; then
    echo -e "${RED}❌ Device not found (404)${NC}"
    echo "Check that the device ID and model are correct"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

elif [ "$HTTP_CODE" = "400" ]; then
    echo -e "${RED}❌ Bad request (400)${NC}"
    echo "The command format may be incorrect for this device"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

else
    echo -e "${RED}❌ Request failed (${HTTP_CODE})${NC}"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    exit 1
fi
