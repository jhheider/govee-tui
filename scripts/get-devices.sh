#!/bin/bash
# Test script to fetch devices from Govee API
# Usage: ./scripts/get-devices.sh [API_KEY]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get API key from argument, env, or config file
API_KEY="${1:-${GOVEE_API_KEY}}"
if [ -z "$API_KEY" ]; then
    CONFIG_FILE="$HOME/.config/govee-tui/config.toml"
    if [ -f "$CONFIG_FILE" ]; then
        API_KEY=$(grep '^key = ' "$CONFIG_FILE" | cut -d'"' -f2)
    fi
fi

if [ -z "$API_KEY" ] || [ "$API_KEY" = "YOUR_API_KEY_HERE" ]; then
    echo -e "${RED}❌ Error: No API key provided${NC}"
    echo ""
    echo "Usage:"
    echo "  $0 <API_KEY>"
    echo "  GOVEE_API_KEY=xxx $0"
    echo "  # Or set key in ~/.config/govee-tui/config.toml"
    exit 1
fi

# Check dependencies
command -v curl >/dev/null 2>&1 || { echo -e "${RED}❌ curl is required${NC}"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo -e "${RED}❌ jq is required${NC}"; exit 1; }

echo -e "${BLUE}📡 Fetching devices from Govee API...${NC}"
echo ""

# API endpoint (try new router endpoint first)
BASE_URL="https://openapi.api.govee.com"
ENDPOINT="/router/api/v1/user/devices"

echo -e "${YELLOW}Endpoint:${NC} ${BASE_URL}${ENDPOINT}"
echo -e "${YELLOW}API Key:${NC} ${API_KEY:0:8}...${API_KEY: -4}"
echo ""

# Make request
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
    "${BASE_URL}${ENDPOINT}" \
    -H "Govee-API-Key: ${API_KEY}" \
    -H "Content-Type: application/json")

# Split response and status code
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo -e "${YELLOW}HTTP Status:${NC} ${HTTP_CODE}"
echo ""

# Check status code
if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Success!${NC}"
    echo ""

    # Parse and display device count
    DEVICE_COUNT=$(echo "$BODY" | jq -r '.data | length' 2>/dev/null || echo "0")
    echo -e "${GREEN}Found ${DEVICE_COUNT} device(s)${NC}"
    echo ""

    # Show device list in table format
    echo -e "${YELLOW}Devices:${NC}"
    echo "$BODY" | jq -r '.data[] |
        "\(.sku) | \(.device) | \(.deviceName)"' |
        awk -F' \\| ' 'BEGIN {printf "%-20s | %-30s | %s\n", "Model", "ID", "Name"; print "-------------------------------------------------------------------"}
        {printf "%-20s | %-30s | %s\n", $1, $2, $3}'
    echo ""

    # Show full JSON for first device as example
    echo -e "${YELLOW}Sample Device (full JSON):${NC}"
    echo "$BODY" | jq '.data[0]' 2>/dev/null || echo "No devices found"
    echo ""

    # Show capabilities summary
    echo -e "${YELLOW}Capabilities Summary:${NC}"
    echo "$BODY" | jq -r '.data[] |
        "\(.deviceName): \(.capabilities | length) capabilities - \([.capabilities[].type] | join(", "))"' | head -5
    echo ""

    # Save raw response
    echo "$BODY" | jq . > /tmp/govee-devices.json
    echo -e "${BLUE}💾 Raw JSON saved to /tmp/govee-devices.json${NC}"

elif [ "$HTTP_CODE" = "401" ]; then
    echo -e "${RED}❌ Authentication failed (401)${NC}"
    echo "Your API key is invalid or expired"
    echo ""
    echo "Response:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

elif [ "$HTTP_CODE" = "429" ]; then
    echo -e "${RED}❌ Rate limit exceeded (429)${NC}"
    echo "You've hit the daily request limit (10,000/day)"
    echo ""
    echo "Response:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

else
    echo -e "${RED}❌ Request failed (${HTTP_CODE})${NC}"
    echo ""
    echo "Response:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    exit 1
fi

# Try legacy endpoint if new one didn't work
if [ "$HTTP_CODE" != "200" ]; then
    echo ""
    echo -e "${YELLOW}Trying legacy endpoint: /v1/devices${NC}"

    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        "${BASE_URL}/v1/devices" \
        -H "Govee-API-Key: ${API_KEY}" \
        -H "Content-Type: application/json")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Legacy endpoint works!${NC}"
        echo "$BODY" | jq .
    fi
fi
