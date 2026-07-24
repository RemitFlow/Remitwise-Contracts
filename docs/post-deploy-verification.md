# Post-Deploy Verification Checklist

This document provides a step-by-step checklist to verify that a deployed RemitFlow contract on Testnet or Mainnet is properly initialized, configured, and operational.

---

## 1. Bytecode & WASM Hash Verification

- [ ] **Locally compiled WASM binary hash match**: Run `./scripts/verify-wasm-hash.sh <network> <CONTRACT_ID>` to ensure on-chain bytecode matches local release build hash.
- [ ] **WASM Size Budget**: Confirm WASM binary size does not exceed the 64 KB limit (`65,536` bytes).
- [ ] **Reproducible Build Check**: Verify local build toolchain uses standard target `wasm32-unknown-unknown` under release profile.

---

## 2. Initialization & Contract Storage Check

- [ ] **Admin Address Verification**: Invoke `get_admin` to verify the returned address matches the intended administrator address:
  ```sh
  stellar contract invoke --id <CONTRACT_ID> --network <NETWORK> -- get_admin
  ```
- [ ] **Token Address Verification**: Invoke `get_token` to confirm the configured Soroban token contract address:
  ```sh
  stellar contract invoke --id <CONTRACT_ID> --network <NETWORK> -- get_token
  ```
- [ ] **Transfer Counter Initialization**: Invoke `counter` to verify the counter starts at `0`.
- [ ] **Pause State Check**: Invoke `is_paused` to confirm initial pause status is `false`.
- [ ] **Re-initialization Rejection**: Attempt calling `initialize` a second time and verify it returns `Error::AlreadyInitialized`.

---

## 3. Privileged Callers Allowlist Check

- [ ] **Allowlist Query**: Check designated sender addresses via `is_caller_allowed(caller)`.
- [ ] **Add Callers**: Admin populates initial allowlist of operational senders via `add_caller(caller)` and confirms `caller_added` event emission.
- [ ] **Unauthorized Caller Gating**: Confirm non-allowlisted addresses are rejected with `Error::CallerNotAllowed` when invoking `create_transfer`.

---

## 4. End-to-End Escrow Lifecycle Smoke Test

- [ ] **Create Transfer**: Submit a small test transfer via `create_transfer(from, recipient, amount, expiry)` from an allowlisted sender.
  - Confirm transfer ID is assigned (e.g. `1`).
  - Confirm `created` event is emitted.
  - Verify total escrow balance updates (`total_escrowed`).
- [ ] **Claim Transfer**: Execute `claim_transfer(id, recipient)` using recipient authorization.
  - Verify recipient receives tokens.
  - Confirm transfer status becomes `Claimed`.
  - Confirm `claimed` event is emitted.
- [ ] **Cancel Transfer (Expiry Test)**: Create a short-expiry transfer, wait for ledger expiry, and invoke `cancel_transfer(id, from)`.
  - Verify sender receives refund.
  - Confirm transfer status becomes `Cancelled`.
  - Confirm `cancelled` event is emitted.

---

## 5. Admin Authorization & Emergency Controls Verification

- [ ] **Admin Pause**: Call `pause()` as admin and verify `is_paused` returns `true` and new transfer creations are blocked with `Error::ContractPaused`.
- [ ] **Admin Unpause**: Call `unpause()` as admin and verify creation of new transfers is restored.
- [ ] **Non-Admin Rejection**: Attempt admin operations (`pause`, `add_caller`, `transfer_admin`) using a non-admin key and verify rejection.

---

## 6. Observability & Event Indexer Verification

- [ ] **Event Topics Alignment**: Ensure off-chain indexer/monitoring system captures emitted events:
  - `init`
  - `created`
  - `claimed`
  - `cancelled`
  - `paused`
  - `unpaused`
  - `caller_added`
  - `caller_removed`
  - `admin_transfer_started`
  - `admin_transfer_completed`
- [ ] **TTL & Storage Expiry Monitoring**: Verify instance TTL extension is active on mutating calls.
