#!/bin/bash
# Wisdoverse Nexus E2E Test Script
# Usage: ./scripts/e2e_test.sh [BASE_URL] [TOKEN]

set -e

BASE_URL="${1:-http://localhost:8081}"
TOKEN="$2"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}✅ $1${NC}"; }
fail() { echo -e "${RED}❌ $1${NC}"; exit 1; }

echo "=========================================="
echo "   Wisdoverse Nexus E2E Test"
echo "   URL: $BASE_URL"
echo "=========================================="

# 1. Health
echo ""
echo "=== 1. Health Check ==="
HEALTH_RESPONSE=$(curl -sS --max-time 5 -w $'\n%{http_code}' "$BASE_URL/health" 2>/dev/null || true)
HEALTH_STATUS=$(printf '%s\n' "$HEALTH_RESPONSE" | tail -n1)
HEALTH_BODY=$(printf '%s\n' "$HEALTH_RESPONSE" | sed '$d' | tr -d '\r' | xargs)

[ "$HEALTH_STATUS" = "200" ] && [ "$HEALTH_BODY" = "OK" ] && pass "Health check" || fail "Health check failed (status=$HEALTH_STATUS, body='$HEALTH_BODY')"

# 2. Auth protection
echo ""
echo "=== 2. Auth Protection ==="
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/v1/rooms" \
  -H "Content-Type: application/json" -d '{"name":"x"}')
[ "$STATUS" = "401" ] && pass "Auth protection (401)" || fail "Expected 401, got $STATUS"

# 3. Create room
echo ""
echo "=== 3. Create Room ==="
RESPONSE=$(curl -sf -X POST "$BASE_URL/v1/rooms" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"E2E Test Room","topic":"Automated testing"}')
ROOM_ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
[ -n "$ROOM_ID" ] && pass "Created room: $ROOM_ID" || fail "Create room failed"

# 4. Send message
echo ""
echo "=== 4. Send Message ==="
RESPONSE=$(curl -sf -X POST "$BASE_URL/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"roomId\":\"$ROOM_ID\",\"sender\":\"e2e-test\",\"text\":\"Test message\"}")
MSG_ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
[ -n "$MSG_ID" ] && pass "Sent message: $MSG_ID" || fail "Send message failed"

# 5. Get room
echo ""
echo "=== 5. Get Room ==="
RESPONSE=$(curl -sf "$BASE_URL/v1/rooms/$ROOM_ID" \
  -H "Authorization: Bearer $TOKEN")
echo "$RESPONSE" | grep -q "$MSG_ID" && pass "Room contains message" || fail "Message not found"

echo ""
echo "=========================================="
echo -e "${GREEN}✅ All E2E tests passed!${NC}"
echo "=========================================="
