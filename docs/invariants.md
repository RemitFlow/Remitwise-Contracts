# Invariants

This note documents the invariants the RemitFlow contract relies on for
correctness, and how each one is enforced or checked.

---

## Escrow-Conservation Invariant

**Every amount accepted into escrow must remain either pending or have been
released exactly once.**

```
TotalFunded = TotalEscrowed + TotalReleased
```

`TotalEscrowed` is the current pending liability. `TotalFunded` and
`TotalReleased` are monotonic lifetime totals, so a terminal transition cannot
silently erase the amount that was originally funded. The token balance must
also cover the pending liability:

```
token_balance(contract_address) >= TotalEscrowed
```

The three accounting values are maintained incrementally by the
`EscrowAccounting` module. This keeps lifecycle operations O(1) and gives all
funding and release paths one owner for their arithmetic.

### Why this can drift

Bookkeeping bugs can miss or double-apply an update to one of the totals.
Non-standard tokens can also credit the escrow with less than the amount
requested, leaving the contract unable to satisfy its pending liability.
Either case could let the contract accept and account for more than it can
actually pay out.

### Enforcement

`EscrowAccounting::assert_invariant` checks the conservation equation and the
token balance, returning [`Error::SupplyInvariantViolation`](./error-reference.md)
if either diverges. It runs after the accounting and token updates in every
entrypoint that moves funds:

- `create_transfer`
- `claim_transfer`
- `cancel_transfer`
- `sweep_expired`

Funding and release arithmetic is checked before token movement and repeated
when the state transition is recorded. Overflow and under-release attempts
return [`Error::AccountingOverflow`](./error-reference.md). Because a
`Result::Err` returned from a Soroban entrypoint rolls back the entire
invocation, a failed lifecycle operation cannot leave a token movement without
its matching accounting update. `batch_operations` inherits the same
protection through its delegated lifecycle calls.

The checks are also exposed through the public, read-only
[`check_supply_invariant`](./entrypoint-reference.md#check_supply_invariant-result-error)
entrypoint so off-chain monitoring can audit solvency and conservation.

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
5. **Terminal releases are one-for-one.** Claims, sender cancellations, and
   permissionless expiry sweeps all pass through the same checked release
   transition, so each pending record can reduce escrow exactly once.

## See Also

- [Storage Model](./storage-model.md) — key layout backing `TotalEscrowed`
- [Error Reference](./error-reference.md) — full error code table
- [Entrypoint Reference](./entrypoint-reference.md) — per-entrypoint interface docs
