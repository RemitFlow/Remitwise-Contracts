# TTL and Rent Management

This document describes the Time-To-Live (TTL) bump strategy, state rent management, and automated test validation for the **RemitFlow** smart contract on Soroban.

---

## Overview

Soroban requires state entries to maintain a positive Time-To-Live (TTL) to remain active on-chain. Entries whose TTL expires are archived. RemitFlow manages state TTL across two distinct storage tiers: **Instance Storage** and **Persistent Storage**.

---

## Storage Tiers & TTL Bump Configuration

| Storage Tier | Storage Enum | Threshold (`ledgers`) | Bump Amount (`ledgers`) | Bumping Triggers |
|--------------|--------------|-----------------------|-------------------------|------------------|
| **Instance** | `InstanceKey` | `518_400` (≈ 30 days) | `535_680` (≈ 31 days) | All mutating contract calls (`initialize`, `pause`, `unpause`, `add_caller`, `remove_caller`, `create_transfer`, `claim_transfer`, `cancel_transfer`, `transfer_admin`, `accept_admin`) |
| **Persistent** | `PersistentKey::Transfer(id)` | `518_400` (≈ 30 days) | `535_680` (≈ 31 days) | Entry creation and state updates (`set_transfer` during `create_transfer`, `claim_transfer`, `cancel_transfer`) |
| **Persistent** | `PersistentKey::AllowedCaller(addr)` | `518_400` (≈ 30 days) | `535_680` (≈ 31 days) | Allowlist additions (`set_caller_allowed` during `add_caller`) |

---

## Constants in `src/storage.rs`

- `INSTANCE_BUMP_THRESHOLD`: `518_400` ledgers
- `INSTANCE_BUMP_AMOUNT`: `535_680` ledgers
- `PERSISTENT_BUMP_THRESHOLD`: `518_400` ledgers
- `PERSISTENT_BUMP_AMOUNT`: `535_680` ledgers

---

## Automated Test Verification

Storage TTL bump behavior is validated in `src/test.rs` via automated tests:

- **`test_ttl_bump_constants_configured_correctly`**: Confirms that constants match configured limits and satisfy `amount >= threshold`.
- **`test_instance_ttl_bumped_on_mutating_calls`**: Verifies `extend_instance` and instance TTL refreshes across contract state transitions.
- **`test_persistent_ttl_bumped_on_transfer_and_caller_writes`**: Validates persistent TTL extensions when writing `AllowedCaller` and `Transfer` entries.
- **`test_cancel_transfer_bumps_persistent_ttl`**: Verifies persistent TTL extensions when updating transfer status on cancellation.
- **`test_admin_transfer_flow_bumps_instance_ttl`**: Confirms instance TTL bumps during two-step admin ownership transfers.
- **`test_storage_ttl_expiration_behavior`**: Validates persistent storage lookup and positive TTL status for active records.

See `docs/storage-model.md` and `src/storage.rs` for implementation details.

