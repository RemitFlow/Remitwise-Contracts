#!/usr/bin/env sh

set -eu

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT HUP INT TERM

# A file exactly at the limit is valid.
dd if=/dev/zero of="$TMP_DIR/at-budget.wasm" bs=1 count=16 2>/dev/null
WASM_SIZE_BUDGET_BYTES=16 ./scripts/check-wasm-size.sh "$TMP_DIR/at-budget.wasm"

# One byte above the limit must fail.
dd if=/dev/zero of="$TMP_DIR/over-budget.wasm" bs=1 count=17 2>/dev/null
if WASM_SIZE_BUDGET_BYTES=16 ./scripts/check-wasm-size.sh "$TMP_DIR/over-budget.wasm"; then
  echo "error: oversized fixture unexpectedly passed" >&2
  exit 1
fi

# Missing artifacts and invalid configuration must also fail clearly.
if WASM_SIZE_BUDGET_BYTES=16 ./scripts/check-wasm-size.sh "$TMP_DIR/missing.wasm"; then
  echo "error: missing fixture unexpectedly passed" >&2
  exit 1
fi

if WASM_SIZE_BUDGET_BYTES=invalid ./scripts/check-wasm-size.sh "$TMP_DIR/at-budget.wasm"; then
  echo "error: invalid budget unexpectedly passed" >&2
  exit 1
fi

echo "WASM size checker tests passed"
