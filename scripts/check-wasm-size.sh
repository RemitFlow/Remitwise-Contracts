#!/usr/bin/env sh

set -eu

# Keep this value in sync with docs/resource-limits.md. The limit is expressed
# in bytes so the result is identical on every platform and in CI.
WASM_SIZE_BUDGET_BYTES="${WASM_SIZE_BUDGET_BYTES:-65536}"
WASM_PATH="${1:-target/wasm32-unknown-unknown/release/remitflow_contract.wasm}"

case "$WASM_SIZE_BUDGET_BYTES" in
  ''|*[!0-9]*)
    echo "error: WASM_SIZE_BUDGET_BYTES must be a positive integer" >&2
    exit 2
    ;;
esac

if [ "$WASM_SIZE_BUDGET_BYTES" -eq 0 ]; then
  echo "error: WASM_SIZE_BUDGET_BYTES must be a positive integer" >&2
  exit 2
fi

if [ ! -f "$WASM_PATH" ]; then
  echo "error: WASM artifact not found: $WASM_PATH" >&2
  echo "build it first with: make build" >&2
  exit 2
fi

WASM_SIZE_BYTES=$(wc -c < "$WASM_PATH" | tr -d '[:space:]')

echo "WASM size: ${WASM_SIZE_BYTES} bytes (budget: ${WASM_SIZE_BUDGET_BYTES} bytes)"

if [ "$WASM_SIZE_BYTES" -gt "$WASM_SIZE_BUDGET_BYTES" ]; then
  OVER_BUDGET_BYTES=$((WASM_SIZE_BYTES - WASM_SIZE_BUDGET_BYTES))
  echo "error: $WASM_PATH exceeds the WASM size budget by ${OVER_BUDGET_BYTES} bytes" >&2
  exit 1
fi

REMAINING_BYTES=$((WASM_SIZE_BUDGET_BYTES - WASM_SIZE_BYTES))
echo "WASM size check passed (${REMAINING_BYTES} bytes remaining)"
