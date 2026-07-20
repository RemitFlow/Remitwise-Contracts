# Error Reference

Every error returned by `remitflow-contract` is a `#[contracterror]` enum
variant serialised as a `u32`. Soroban returns this numeric code to the
caller; the table below maps each code back to the Rust identifier, its
meaning, and the entrypoints that can produce it.

---

## Error Codes

| Code | Variant | Description | Returned By |
|------|---------|-------------|-------------|
| 1 | `AlreadyInitialized` | The contract has already been initialised with an admin and token address. Second calls to `initialize` are rejected. | `initialize` |
| 2 | `NotInitialized` | The contract has not been initialised yet. Most public entrypoints require initialisation before they can proceed. | `initialize`, `get_admin`, `get_token`, `pause`, `unpause`, `create_transfer`, `claim_transfer`, `cancel_transfer`, `add_caller`, `remove_caller` |
| 3 | `TransferNotFound` | No record exists for the supplied transfer `id`. Either the id was never created or it was assigned to a transfer that was purged. | `get_transfer`, `get_status`, `transfer_exists`, `is_expired`, `claim_transfer`, `cancel_transfer` |
| 4 | `InvalidAmount` | The supplied transfer `amount` is not strictly positive (zero or negative). | `create_transfer` |
| 5 | `InvalidExpiry` | The supplied `expiry` timestamp is not in the future — it must be strictly greater than the current ledger timestamp. | `create_transfer` |
| 6 | `CounterOverflow` | The transfer counter would overflow its `u64` range. The contract supports at most `u64::MAX` transfers in its lifetime. | `create_transfer` |
| 7 | `Unauthorized` | The caller is not the expected party for this action. Raised when the recipient does not match on `claim_transfer` or the sender does not match on `cancel_transfer`. | `claim_transfer`, `cancel_transfer` |
| 8 | `NotPending` | The transfer is not in the `Pending` state, so it cannot be claimed or cancelled. Already-claimed or cancelled transfers are final. | `claim_transfer`, `cancel_transfer` |
| 9 | `Expired` | The transfer has passed its expiry deadline and can no longer be claimed by the recipient. | `claim_transfer` |
| 10 | `NotExpired` | The transfer has not yet reached its expiry timestamp, so the sender cannot cancel it yet. | `cancel_transfer` |
| 11 | `SameParty` | The `from` and `recipient` addresses are identical. The contract requires the sender and recipient to be different. | `create_transfer` |
| 12 | `AmountTooLarge` | The supplied `amount` exceeds [`MAX_AMOUNT`](https://github.com/RemitFlow/Remitwise-Contracts/blob/main/src/lib.rs) (1,000,000,000,000,000,000). | `create_transfer` |
| 13 | `ContractPaused` | The contract is currently paused by the administrator. New transfers cannot be created while paused, but claims and cancellations of existing transfers remain available. | `create_transfer` |
| 14 | `ExpiryTooFar` | The supplied `expiry` is further in the future than [`MAX_EXPIRY_WINDOW`](https://github.com/RemitFlow/Remitwise-Contracts/blob/main/src/lib.rs) (~1 year / 31,536,000 seconds) from now. | `create_transfer` |
| 15 | `EscrowCapReached` | Accepting this transfer would push the total escrowed balance above [`MAX_TOTAL_ESCROWED`](https://github.com/RemitFlow/Remitwise-Contracts/blob/main/src/lib.rs). | `create_transfer` |
| 16 | `CallerNotAllowed` | The `from` address is not on the privileged callers allowlist. Only allowlisted addresses may create transfers. | `create_transfer` |

---

## Entrypoint → Error Map

| Entrypoint | Possible Errors |
|---|---|
| `initialize` | `AlreadyInitialized` (1)* |
| `get_admin`, `get_token` | `NotInitialized` (2) |
| `counter`, `is_paused`, `is_caller_allowed` | None† |
| `pause`, `unpause` | `NotInitialized` (2) |
| `add_caller`, `remove_caller` | `NotInitialized` (2) |
| `create_transfer` | `NotInitialized` (2), `ContractPaused` (13), `CallerNotAllowed` (16), `InvalidAmount` (4), `AmountTooLarge` (12), `EscrowCapReached` (15), `InvalidExpiry` (5), `ExpiryTooFar` (14), `SameParty` (11), `CounterOverflow` (6) |
| `claim_transfer` | `NotInitialized` (2), `TransferNotFound` (3), `Unauthorized` (7), `NotPending` (8), `Expired` (9) |
| `cancel_transfer` | `NotInitialized` (2), `TransferNotFound` (3), `Unauthorized` (7), `NotPending` (8), `NotExpired` (10) |
| `get_transfer`, `transfer_exists`, `get_status`, `is_expired` | `NotInitialized` (2), `TransferNotFound` (3)‡ |
| `total_escrowed`, `count_for_sender`, `count_for_recipient`, `count_by_status`, `get_transfers_paged` | None† |
| `batch_operations` | Any error from `create_transfer`, `claim_transfer`, or `cancel_transfer` depending on the operations in the batch |

\* `initialize` does not return `NotInitialized` (it is the call that performs initialisation).  
† Returns `Ok` / plain value instead of `Result`; no error path.  
‡ `transfer_exists` returns `bool` (no error path); `is_expired` returns `NotInitialized` (2) or `TransferNotFound` (3).

---

## How Errors Reach the Caller

Soroban converts a `Result::Err(Error)` into a failed invocation. The numeric
code surfaces in the transaction result envelope:

```
{
  "error": "ContractError",
  "contract_error_code": 7
}
```

Code `7` above means `Unauthorized`. Integrators should map these codes back
using the table at the top of this page.

## See Also

- [`src/error.rs`](https://github.com/RemitFlow/Remitwise-Contracts/blob/main/src/error.rs) — authoritative source for the enum definition
- [Entrypoint Reference](./entrypoint-reference.md) — detailed interface docs per entrypoint
