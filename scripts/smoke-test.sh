#!/bin/bash
# Smoke test for RemitFlow testnet deployment
set -euo pipefail

NETWORK="${1:-testnet}"
SOURCE="${2:-deployer}"

echo "Building WASM..."
cargo build --target wasm32-unknown-unknown --release

WASM="target/wasm32-unknown-unknown/release/remitflow_contract.wasm"
echo "Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy --wasm "$WASM" --source "$SOURCE" --network "$NETWORK")
echo "Contract deployed: $CONTRACT_ID"

echo "Initializing..."
stellar contract invoke --id "$CONTRACT_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$SOURCE" --token "$CONTRACT_ID"

echo "Verifying admin..."
ADMIN=$(stellar contract invoke --id "$CONTRACT_ID" --source "$SOURCE" --network "$NETWORK" -- get_admin)
echo "Admin: $ADMIN"

echo "Smoke test passed!"
