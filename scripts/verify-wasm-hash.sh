#!/bin/bash
# verify-wasm-hash.sh - Verify deployed WASM hash matches local build
set -euo pipefail
WASM_PATH="target/wasm32-unknown-unknown/release/remitflow_contract.wasm"
NETWORK="${1:-testnet}"
CONTRACT_ID="${2:-}"

# Build if not present
[ -f "$WASM_PATH" ] || cargo build --target wasm32-unknown-unknown --release

# Compute local hash
LOCAL_HASH=$(sha256sum "$WASM_PATH" | awk "{print \$1}")
echo "Local WASM hash:  $LOCAL_HASH"

# Compare with deployed if contract ID provided
if [ -n "$CONTRACT_ID" ]; then
  echo "Fetching on-chain hash for $CONTRACT_ID on $NETWORK..."
  ONCHAIN_HASH=$(stellar contract inspect --network "$NETWORK" --id "$CONTRACT_ID" --output json 2>/dev/null | grep -o "\"wasm_hash\":\"[^\"]*\"" | cut -d"\"" -f4)
  echo "On-chain hash:     $ONCHAIN_HASH"
  if [ "$LOCAL_HASH" = "$ONCHAIN_HASH" ]; then
    echo "OK: WASM hash matches"
  else
    echo "MISMATCH: Local=$LOCAL_HASH On-chain=$ONCHAIN_HASH"
    exit 1
  fi
else
  echo "Run with: ./scripts/verify-wasm-hash.sh testnet <contract-id>"
fi
