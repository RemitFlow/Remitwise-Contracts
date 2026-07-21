# Deployment Guide

This guide explains how to build, deploy, and initialize the RemitFlow contract
on a Stellar network using the automated script or the Stellar CLI directly.

---

## Prerequisites

| Tool | Install |
|---|---|
| Rust + `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | `cargo install --locked stellar-cli` |
| A funded Stellar identity | `stellar keys generate --global <name> --network testnet --fund` |

Confirm the CLI is available:

```sh
stellar --version
```

---

## Quick Start — Automated Script

The [`scripts/deploy-and-initialize.sh`](../scripts/deploy-and-initialize.sh)
script performs the full build → deploy → initialize sequence in one command.

```sh
./scripts/deploy-and-initialize.sh \
  --network  testnet \
  --source   my-key \
  --admin    GABC...XYZ \
  --token    CABC...XYZ
```

Or via `make`:

```sh
make deploy \
  NETWORK=testnet \
  SOURCE=my-key \
  ADMIN=GABC...XYZ \
  TOKEN=CABC...XYZ
```

The script prints a summary on success, including the assigned contract ID and
the next steps for adding privileged callers.

### Script Options

| Flag | Env var | Default | Description |
|---|---|---|---|
| `--network <name>` | — | `testnet` | Stellar network profile |
| `--source <name>` | `STELLAR_SOURCE` | `deployer` | Signing identity alias |
| `--admin <address>` | `ADMIN_ADDRESS` | *(required)* | Address set as admin in `initialize()` |
| `--token <address>` | `TOKEN_ADDRESS` | *(required)* | Soroban token contract for escrow |
| `--wasm <path>` | — | `target/…/remitflow_contract.wasm` | Override the WASM path |
| `--skip-build` | — | off | Skip `cargo build` (use existing WASM) |

### Environment variables

Credentials and addresses can be supplied via environment variables instead of
flags, which is useful in CI or when sourcing a `.env` file:

```sh
export STELLAR_SOURCE=my-key
export ADMIN_ADDRESS=GABC...XYZ
export TOKEN_ADDRESS=CABC...XYZ
./scripts/deploy-and-initialize.sh --network testnet
```

> [!CAUTION]
> Never commit secrets or private key material to version control. Use
> `stellar keys generate` to manage identities; the private key is stored
> in your local Stellar config directory.

---

## Manual Steps (Stellar CLI)

If you prefer to run each step individually:

### 1. Build

```sh
make build
# or:
cargo build --target wasm32-unknown-unknown --release
```

The compiled artifact is written to:
```
target/wasm32-unknown-unknown/release/remitflow_contract.wasm
```

### 2. Deploy

```sh
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/remitflow_contract.wasm \
  --source <KEY_ALIAS> \
  --network testnet
```

The command outputs the contract ID. Save it for subsequent calls.

### 3. Initialize

```sh
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <KEY_ALIAS> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS>
```

`initialize` can only be called once. Subsequent calls return
`AlreadyInitialized`.

### 4. Add privileged callers (optional)

Only addresses on the allowlist can create transfers:

```sh
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_KEY_ALIAS> \
  --network testnet \
  -- add_caller \
  --caller <CALLER_ADDRESS>
```

|---|

## Post-Deploy Verification

Use the bundled hash verification script to confirm the deployed WASM matches
your local build:

```sh
./scripts/verify-wasm-hash.sh testnet <CONTRACT_ID>
```

For a full walkthrough of the deterministic build settings, prerequisites, and
troubleshooting steps, see the [Reproducible Builds guide](reproducible-build.md).

|---|

## Mainnet Deployment

The same script and steps apply to mainnet. Change `--network testnet` to
`--network mainnet` and ensure:

- The signing identity uses a hardware wallet or multisig for the admin key.
- The contract has been audited and all tests pass on testnet first.
- See [`docs/mainnet-checklist.md`](mainnet-checklist.md) for the full
  pre-launch checklist.

---

## Network Profiles

The Stellar CLI uses named network profiles. Common built-in profiles:

| Profile | RPC | Description |
|---|---|---|
| `testnet` | `https://soroban-testnet.stellar.org` | Public testnet (reset periodically) |
| `mainnet` | `https://mainnet.sorobanrpc.com` | Public mainnet |
| `local` | `http://localhost:8000/soroban/rpc` | `stellar network start local` |

List configured profiles:

```sh
stellar network ls
```
