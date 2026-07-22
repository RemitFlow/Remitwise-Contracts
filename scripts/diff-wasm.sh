#!/bin/bash
# Compare two WASM builds and report differences
set -euo pipefail

WASM1="${1:-target/wasm32-unknown-unknown/release/remitflow_contract.wasm}"
WASM2="${2:-}"

if [ -z "$WASM2" ]; then
  echo "Usage: ./scripts/diff-wasm.sh <wasm1> <wasm2>"
  exit 1
fi

echo "Comparing:"
echo "  WASM 1: $WASM1 ($(wc -c < "$WASM1") bytes)"
echo "  WASM 2: $WASM2 ($(wc -c < "$WASM2") bytes)"

HASH1=$(sha256sum "$WASM1" | cut -d" " -f1)
HASH2=$(sha256sum "$WASM2" | cut -d" " -f1)

if [ "$HASH1" = "$HASH2" ]; then
  echo "WASM files are identical (same sha256)"
  exit 0
fi

echo "SHA256 differs:"
echo "  WASM 1: $HASH1"
echo "  WASM 2: $HASH2"
echo "Size difference: $(($(wc -c < "$WASM2") - $(wc -c < "$WASM1"))) bytes"
