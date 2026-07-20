#!/usr/bin/env bash
# deploy-and-initialize.sh
#
# Build, deploy, and initialize the RemitFlow contract in one step.
#
# Usage:
#   ./scripts/deploy-and-initialize.sh [OPTIONS]
#
# Options:
#   --network   <name>     Stellar network profile (default: testnet)
#   --source    <name>     Stellar identity / key alias to sign transactions
#                          (default: value of STELLAR_SOURCE env var, or "deployer")
#   --admin     <address>  Address that will be set as the contract admin
#                          (default: value of ADMIN_ADDRESS env var)
#   --token     <address>  Soroban token contract address to use for escrow
#                          (default: value of TOKEN_ADDRESS env var)
#   --wasm      <path>     Path to the pre-built WASM file
#                          (default: target/wasm32-unknown-unknown/release/remitflow_contract.wasm)
#   --skip-build           Skip the cargo build step (use existing WASM)
#   --help                 Print this help message and exit
#
# Environment variables (can substitute for flags):
#   STELLAR_SOURCE   Stellar identity alias
#   ADMIN_ADDRESS    Admin address for initialize()
#   TOKEN_ADDRESS    Token contract address for initialize()
#
# Examples:
#   # Full deploy to testnet with all options as flags
#   ./scripts/deploy-and-initialize.sh \
#     --network testnet \
#     --source my-key \
#     --admin GABC...XYZ \
#     --token CABC...XYZ
#
#   # Use environment variables (e.g. from a .env file)
#   export STELLAR_SOURCE=my-key
#   export ADMIN_ADDRESS=GABC...XYZ
#   export TOKEN_ADDRESS=CABC...XYZ
#   ./scripts/deploy-and-initialize.sh --network testnet
#
#   # Skip build if WASM is already compiled
#   ./scripts/deploy-and-initialize.sh --skip-build --network testnet \
#     --source my-key --admin GABC...XYZ --token CABC...XYZ

set -euo pipefail

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
NETWORK="testnet"
SOURCE="${STELLAR_SOURCE:-deployer}"
ADMIN="${ADMIN_ADDRESS:-}"
TOKEN="${TOKEN_ADDRESS:-}"
WASM_PATH="target/wasm32-unknown-unknown/release/remitflow_contract.wasm"
SKIP_BUILD=false

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
usage() {
    sed -n '3,57p' "$0" | sed 's/^# \?//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --network)   NETWORK="$2";    shift 2 ;;
        --source)    SOURCE="$2";     shift 2 ;;
        --admin)     ADMIN="$2";      shift 2 ;;
        --token)     TOKEN="$2";      shift 2 ;;
        --wasm)      WASM_PATH="$2";  shift 2 ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --help|-h)   usage ;;
        *) echo "Unknown option: $1" >&2; usage ;;
    esac
done

# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------
if [[ -z "$ADMIN" ]]; then
    echo "ERROR: --admin (or ADMIN_ADDRESS) is required." >&2
    exit 1
fi
if [[ -z "$TOKEN" ]]; then
    echo "ERROR: --token (or TOKEN_ADDRESS) is required." >&2
    exit 1
fi
if [[ -z "$SOURCE" ]]; then
    echo "ERROR: --source (or STELLAR_SOURCE) is required." >&2
    exit 1
fi

if ! command -v stellar &>/dev/null; then
    echo "ERROR: 'stellar' CLI not found. Install with: cargo install --locked stellar-cli" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Step 1 — Build
# ---------------------------------------------------------------------------
if [[ "$SKIP_BUILD" == "false" ]]; then
    echo "==> Building WASM..."
    cargo build --target wasm32-unknown-unknown --release
    echo "    Done: $WASM_PATH"
else
    echo "==> Skipping build (--skip-build)"
fi

if [[ ! -f "$WASM_PATH" ]]; then
    echo "ERROR: WASM not found at $WASM_PATH" >&2
    echo "       Run without --skip-build, or pass --wasm <path>." >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Step 2 — Deploy
# ---------------------------------------------------------------------------
echo ""
echo "==> Deploying contract to $NETWORK (source: $SOURCE)..."
CONTRACT_ID=$(
    stellar contract deploy \
        --wasm "$WASM_PATH" \
        --source "$SOURCE" \
        --network "$NETWORK" \
        2>&1
)

# stellar contract deploy prints the contract ID to stdout; capture it
# Strip any trailing whitespace / newlines
CONTRACT_ID="$(echo "$CONTRACT_ID" | tr -d '[:space:]')"

if [[ -z "$CONTRACT_ID" ]]; then
    echo "ERROR: Deploy returned an empty contract ID. Check the stellar CLI output above." >&2
    exit 1
fi

echo "    Contract ID: $CONTRACT_ID"

# ---------------------------------------------------------------------------
# Step 3 — Initialize
# ---------------------------------------------------------------------------
echo ""
echo "==> Initializing contract..."
echo "    Admin : $ADMIN"
echo "    Token : $TOKEN"

stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$SOURCE" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$ADMIN" \
    --token "$TOKEN"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "========================================"
echo "  RemitFlow deployed and initialized"
echo "  Network     : $NETWORK"
echo "  Contract ID : $CONTRACT_ID"
echo "  Admin       : $ADMIN"
echo "  Token       : $TOKEN"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Add privileged callers:"
echo "     stellar contract invoke --id $CONTRACT_ID --source $SOURCE \\"
echo "       --network $NETWORK -- add_caller --caller <ADDRESS>"
echo ""
echo "  2. Verify with verify-wasm-hash.sh:"
echo "     ./scripts/verify-wasm-hash.sh $NETWORK $CONTRACT_ID"
echo ""
echo "  3. Save CONTRACT_ID for future invocations."
