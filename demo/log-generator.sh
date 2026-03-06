#!/usr/bin/env bash
# Sends realistic structured logs to a Better Stack source every second.
#
# Usage:
#   BS_SOURCE_TOKEN=xxx ./demo/log-generator.sh
#
# Run this in one terminal, then `bs logs tail --source <id>` in another.

set -euo pipefail

TOKEN="${BS_SOURCE_TOKEN:?Set BS_SOURCE_TOKEN to your Better Stack source token}"
ENDPOINT="https://in.logs.betterstack.com"

PATHS=("/api/users" "/api/orders" "/api/products" "/api/auth/login" "/api/payments" "/api/webhooks" "/healthz")
SERVICES=("api-gateway" "user-service" "order-service" "payment-service" "auth-service")

uuid() { cat /proc/sys/kernel/random/uuid 2>/dev/null || uuidgen 2>/dev/null || echo "req-$RANDOM-$RANDOM"; }

send() {
  curl -s -o /dev/null "$ENDPOINT" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "$1"
}

now() { date -u +%Y-%m-%dT%H:%M:%S.000Z; }

log_info() {
  local path="${PATHS[$((RANDOM % ${#PATHS[@]}))]}"
  local svc="${SERVICES[$((RANDOM % ${#SERVICES[@]}))]}"
  local methods=("GET" "POST" "PUT")
  local method="${methods[$((RANDOM % 3))]}"
  local status=$((200 + (RANDOM % 2)))
  local dur=$((3 + RANDOM % 180))
  echo "{\"dt\":\"$(now)\",\"level\":\"INFO\",\"message\":\"$method $path $status ${dur}ms\",\"service\":\"$svc\",\"method\":\"$method\",\"path\":\"$path\",\"status\":$status,\"duration_ms\":$dur,\"request_id\":\"$(uuid)\"}"
}

log_warn() {
  local msgs=(
    "Connection pool at 87/100"
    "Request latency exceeded 500ms"
    "Rate limit approaching for 10.0.3.42"
    "Cache miss ratio above 40%"
    "Retry attempt 2/3 for upstream call"
    "SSL certificate expires in 14 days"
  )
  local svc="${SERVICES[$((RANDOM % ${#SERVICES[@]}))]}"
  echo "{\"dt\":\"$(now)\",\"level\":\"WARN\",\"message\":\"${msgs[$((RANDOM % ${#msgs[@]}))]}\",\"service\":\"$svc\",\"request_id\":\"$(uuid)\"}"
}

log_error() {
  local msgs=(
    "Connection refused: payment-service:8443"
    "Query timeout after 30s on orders table"
    "JWT signature verification failed"
    "Database connection pool exhausted"
    "TLS handshake failed: certificate expired"
    "Failed to serialize response"
  )
  local svc="${SERVICES[$((RANDOM % ${#SERVICES[@]}))]}"
  local path="${PATHS[$((RANDOM % ${#PATHS[@]}))]}"
  local status=$((500 + RANDOM % 4))
  local msg="${msgs[$((RANDOM % ${#msgs[@]}))]}"
  echo "{\"dt\":\"$(now)\",\"level\":\"ERROR\",\"message\":\"$msg\",\"service\":\"$svc\",\"path\":\"$path\",\"status\":$status,\"request_id\":\"$(uuid)\"}"
}

log_debug() {
  local msgs=(
    "Cache hit for key users:profile:4821"
    "DNS resolved payment-service.internal in 2ms"
    "Connection reused from pool (idle: 12)"
    "Auth token validated, expires in 3420s"
  )
  local svc="${SERVICES[$((RANDOM % ${#SERVICES[@]}))]}"
  echo "{\"dt\":\"$(now)\",\"level\":\"DEBUG\",\"message\":\"${msgs[$((RANDOM % ${#msgs[@]}))]}\",\"service\":\"$svc\"}"
}

echo "Sending logs to Better Stack (Ctrl+C to stop)..."
echo "Tail with: bs logs tail --source <your-source-id>"
echo ""

n=0
while true; do
  roll=$((RANDOM % 100))
  if   [ $roll -lt 55 ]; then payload=$(log_info)
  elif [ $roll -lt 70 ]; then payload=$(log_debug)
  elif [ $roll -lt 88 ]; then payload=$(log_warn)
  else                        payload=$(log_error)
  fi

  send "$payload"
  n=$((n + 1))
  [ $((n % 10)) -eq 0 ] && echo "  sent $n logs..."
  sleep 1
done
