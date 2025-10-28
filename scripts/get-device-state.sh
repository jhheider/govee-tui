#!/bin/bash
# Test script to get device state from Govee API
# Usage: ./scripts/get-device-state.sh <DEVICE_ID> <MODEL> [API_KEY]

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
API_KEY="${3:-${GOVEE_API_KEY}}"

if [ -z "$DEVICE_ID" ] || [ -z "$MODEL" ]; then
    echo -e "${RED}❌ Error: Missing arguments${NC}"
    echo ""
    echo "Usage:"
    echo "  $0 <DEVICE_ID> <MODEL> [API_KEY]"
    echo ""
    echo "Example:"
    echo "  $0 'AA:BB:CC:DD:EE:FF:11:22' 'H6159'"
    echo "  GOVEE_API_KEY=xxx $0 'AA:BB:CC:DD:EE:FF:11:22' 'H6159'"
    echo ""
    echo "Tip: Run ./scripts/get-devices.sh to see available devices"
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

echo -e "${BLUE}📊 Fetching device state...${NC}"
echo ""

# API endpoint
BASE_URL="https://openapi.api.govee.com"
ENDPOINT="/router/api/v1/device/state"

echo -e "${YELLOW}Device ID:${NC} ${DEVICE_ID}"
echo -e "${YELLOW}Model:${NC} ${MODEL}"
echo ""

# Make request - device state uses query parameters
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
    "${BASE_URL}${ENDPOINT}?device=${DEVICE_ID}&model=${MODEL}" \
    -H "Govee-API-Key: ${API_KEY}" \
    -H "Content-Type: application/json")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo -e "${YELLOW}HTTP Status:${NC} ${HTTP_CODE}"
echo ""

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Success!${NC}"
    echo ""

    # Pretty print full response
    echo -e "${YELLOW}Full Response:${NC}"
    echo "$BODY" | jq .
    echo ""

    # Parse specific state values
    echo -e "${YELLOW}Parsed State:${NC}"

    # Check if we have properties in response
    if echo "$BODY" | jq -e '.data.capabilities' >/dev/null 2>&1; then
        echo "$BODY" | jq -r '.data.capabilities[] |
            "  \(.type // .instance): \(.state.value // .state)"'
    elif echo "$BODY" | jq -e '.data.properties' >/dev/null 2>&1; then
        # Legacy format
        echo "$BODY" | jq -r '.data.properties[] |
            if has("online") then
                "  online: \(.online)"
            elif has("powerState") then
                "  power: \(.powerState)"
            elif has("brightness") then
                "  brightness: \(.brightness)%"
            elif has("color") then
                "  color: RGB(\(.color.r), \(.color.g), \(.color.b))"
            elif has("colorTem") or has("colorTemInKelvin") then
                "  color_temp: \(.colorTem // .colorTemInKelvin)K"
            else
                .
            end'
    else
        echo "$BODY" | jq .
    fi

elif [ "$HTTP_CODE" = "401" ]; then
    echo -e "${RED}❌ Authentication failed (401)${NC}"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

elif [ "$HTTP_CODE" = "404" ]; then
    echo -e "${RED}❌ Device not found (404)${NC}"
    echo "Check that the device ID and model are correct"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

else
    echo -e "${RED}❌ Request failed (${HTTP_CODE})${NC}"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    exit 1
fi

# Try legacy endpoint
if [ "$HTTP_CODE" != "200" ]; then
    echo ""
    echo -e "${YELLOW}Trying legacy endpoint: /v1/devices/state${NC}"

    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        "${BASE_URL}/v1/devices/state?device=${DEVICE_ID}&model=${MODEL}" \
        -H "Govee-API-Key: ${API_KEY}" \
        -H "Content-Type: application/json")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Legacy endpoint works!${NC}"
        echo "$BODY" | jq .
    fi
fi
