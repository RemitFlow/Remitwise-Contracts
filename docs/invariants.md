# Invariants

This note documents the invariants the RemitFlow contract relies on for
correctness, and how each one is enforced or checked.

---

## Supply-Accounting Invariant

**The contract's actual token balance must always be able to cover its
internally tracked escrow liability.**

```
token_balance(contract_address) >= TotalEscrowed
```

### Why this can drift

`TotalEscrowed` ([`InstanceKey::TotalEscrowed`](./storage-model.md)) is a
running total maintained *incrementally*: `create_transfer` adds to it,
`claim_transfer` and `cancel_transfer` subtract from it. It is never
recomputed from the token contract's own balance. That makes every mutating
entrypoint an O(1) operation instead of an O(n) rescan, but it also means the
ledger's belief about how much is escrowed can only be trusted if it is
checked against reality.

Two classes of bug can cause a divergence:

1. **Bookkeeping bugs** in this contract — a missed or double-applied update
   to `TotalEscrowed` that isn't caught by any other check.
2. **Non-standard tokens** — a fee-on-transfer or rebasing token contract
   that credits the escrow with less than the amount requested in a
   `transfer` call, so the contract receives fewer tokens than
   `TotalEscrowed` was incremented by.

Either one, left undetected, could let the contract accept and account for
more than it can actually pay out — an insolvent escrow.

### Enforcement

[`assert_supply_invariant`](https://github.com/RemitFlow/Remitwise-Contracts/blob/main/src/lib.rs)
compares `token::Client::balance(contract_address)` against
`TotalEscrowed` and returns [`Error::SupplyInvariantViolation`](./error-reference.md)
if the balance is less than the tracked liability. It runs automatically,
immediately after the storage and token updates, in every entrypoint that
moves escrowed funds:

- `create_transfer`
- `claim_transfer`
- `cancel_transfer`

Because a `Result::Err` returned from a Soroban entrypoint rolls back the
entire invocation — token movements, storage writes, and events alike — a
violation aborts the operation as if it never happened rather than leaving
the contract in a bad state. `batch_operations` inherits this protection
transitively, since each operation in the batch calls into one of the three
entrypoints above.

The same check is also exposed as a public, read-only entrypoint,
[`check_supply_invariant`](./entrypoint-reference.md#check_supply_invariant-result-error),
so off-chain monitoring can audit contract solvency at any time without
waiting for a mutating call to trip it.

---

## Other Invariants

1. **No two `Transfer` records share an id.** The monotonic counter in
   `InstanceKey::Counter` is incremented via `checked_increment` before
   every `set_transfer` call, and `create_transfer` fails closed with
   `Error::CounterOverflow` rather than wrapping.
2. **A transfer's status only moves forward.** `Pending` transitions to
   exactly one of `Claimed` or `Cancelled`; both `claim_transfer` and
   `cancel_transfer` require the transfer to still be `Pending` before
   acting on it, so a transfer can never be claimed and cancelled, or
   claimed twice.
3. **`from` and `recipient` are always external addresses.**
   `require_external_address` rejects the contract's own address wherever
   an external party address is required (admin, token, sender, recipient,
   allowlisted caller, admin nominee), preventing the contract from being
   configured to hold privileges or funds it cannot exercise.
4. **`TotalEscrowed` never exceeds `MAX_TOTAL_ESCROWED`.** `create_transfer`
   checks the post-increment total against the cap before accepting funds.

## See Also

- [Storage Model](./storage-model.md) — key layout backing `TotalEscrowed`
- [Error Reference](./error-reference.md) — full error code table
- [Entrypoint Reference](./entrypoint-reference.md) — per-entrypoint interface docs
