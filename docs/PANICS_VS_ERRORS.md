# Panics vs Returned Errors

This document describes the RemitFlow contract's philosophy on panics versus
returned errors for contributors, auditors, and operators.

## Rule

**Every public entrypoint returns Result<_, Error>. The contract must never
panic in response to invalid caller input, expired state, or operational
conditions.**

Panics are reserved for unrecoverable internal invariants that indicate a bug
in the contract itself, not in the caller's input.

## What returns errors

Every condition the caller could trigger is surfaced as an Error variant:

| Category | Error variants | When |
| --- | --- | --- |
| Authorization | Unauthorized, CallerNotAllowed | Wrong caller for transfer or allowlist |
| Lifecycle | NotPending, Expired, NotExpired | Transfer in wrong state |
| Validation | InvalidAmount, InvalidExpiry, ExpiryTooFar, SameParty, InvalidAddress | Bad input parameters |
| Limits | AmountTooLarge, EscrowCapReached, AccountLimitReached | Exceeds configured caps |
| State | AlreadyInitialized, NotInitialized, TransferNotFound, ContractPaused, NoPendingAdmin | Contract or transfer state |
| Arithmetic | CounterOverflow | Counter exhausted (u64::MAX) |
| Invariant | SupplyInvariantViolation | Internal bookkeeping vs token balance mismatch |

## What panics

The contract should never panic. If a panic occurs, it is a bug.

Historically, the following patterns are safe because they use unwrap_or
with a sensible default, not unwrap():

- get_counter: defaults to 0 when unset
- get_paused: defaults to false when unset
- get_total_escrowed: defaults to 0 when unset
- is_caller_allowed: defaults to false when unset

## Arithmetic safety

All arithmetic in math.rs returns Option for checked operations.
Callers in lib.rs handle None by returning an appropriate Error:

- checked_add_amount ? Error::AmountTooLarge
- checked_increment ? Error::CounterOverflow

Saturating helpers (saturating_add_amount, saturating_add_with_cap) are
used in read-only tally functions where returning an approximate capped value
is safer than panicking or erroring on a query.

## How to verify

The CI pipeline enforces two invariants:

1. cargo check --workspace must pass with the clippy lints in clippy.toml
   which ban unwrap() and expect() in contract code.
2. scripts/check_no_panic.sh scans for any remaining panic calls.

Run locally:

`sh
make check-no-panic
cargo clippy --workspace -- -D clippy::unwrap_used -D clippy::expect_used
Set-Content -Path "docs\PANICS_VS_ERRORS.md" -Value @"
# Panics vs Returned Errors

This document describes the RemitFlow contract's philosophy on panics versus
returned errors for contributors, auditors, and operators.

## Rule

Every public entrypoint returns Result<_, Error>. The contract must never
panic in response to invalid caller input, expired state, or operational
conditions.

Panics are reserved for unrecoverable internal invariants that indicate a bug
in the contract itself, not in the caller's input.

## What returns errors

Every condition the caller could trigger is surfaced as an Error variant:

Authorization: Unauthorized, CallerNotAllowed — wrong caller
Lifecycle: NotPending, Expired, NotExpired — transfer in wrong state
Validation: InvalidAmount, InvalidExpiry, ExpiryTooFar, SameParty, InvalidAddress — bad input
Limits: AmountTooLarge, EscrowCapReached, AccountLimitReached — exceeds caps
State: AlreadyInitialized, NotInitialized, TransferNotFound, ContractPaused, NoPendingAdmin
Arithmetic: CounterOverflow — counter exhausted
Invariant: SupplyInvariantViolation — bookkeeping vs balance mismatch

## What panics

The contract should never panic. If a panic occurs, it is a bug.

Safe unwrap_or patterns in storage.rs:
- get_counter defaults to 0
- get_paused defaults to false
- get_total_escrowed defaults to 0
- is_caller_allowed defaults to false

## Arithmetic safety

All arithmetic in math.rs returns Option for checked operations.
Callers handle None by returning an appropriate Error:
- checked_add_amount ? Error::AmountTooLarge
- checked_increment ? Error::CounterOverflow

Saturating helpers are used in read-only tally functions where an
approximate capped value is safer than panicking.

## How to verify

CI enforces:
1. clippy lints ban unwrap() and expect() in contract code
2. scripts/check_no_panic.sh scans for panic calls

Run locally:
cargo clippy --workspace -- -D clippy::unwrap_used -D clippy::expect_used

## Adding a new entrypoint

1. Never call unwrap() or expect() in production code
2. Use .ok_or(Error::...) or unwrap_or(default) with documented default
3. Use checked helpers from math.rs, map None to an Error
4. For true invariants, return an Error variant over panicking
