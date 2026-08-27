# Persistent TTL policy

This document describes the Time-To-Live (TTL) bump strategy, state rent
management, cleanup contract, and automated validation for the RemitFlow
smart contract on Soroban. Every state key belongs to one storage tier and one
retention class.

## Design goals

The policy protects three properties that are easy to lose when TTL calls are
spread across entrypoints:

1. A pending transfer must remain readable for the complete operational
   horizon. Expiring its record while escrowed would strand funds or make the
   liability impossible to settle.
2. A write must not keep extending an entry forever merely because a client
   retries a transaction. Bumps are threshold based and monotonic, so a
   healthy entry is not rewritten to a later-than-policy horizon.
3. Maintenance must have predictable cost. Cleanup accepts ids supplied by
   the caller and enforces a hard batch limit; it never scans the transfer
   counter and never removes a pending record.

## Storage tiers and TTL classes

| Storage tier | Key/class | Threshold | Bump horizon | Why |
|---|---|---:|---:|---|
| Instance | `InstanceKey::*` / instance | `518,400` | `535,680` | Admin, token, counters, pause state, and aggregate liability share one contract instance horizon. |
| Persistent | `Transfer(id)` / active | `518,400` | `535,680` | A pending escrow must outlive normal reporting gaps and the maximum transfer expiry window. |
| Persistent | `Transfer(id)` / terminal | `10,080` | `20,160` | Claimed and cancelled records remain queryable for a short audit window, then can be reclaimed. |
| Persistent | `AllowedCaller(address)` | `259,200` | `276,480` | Operational allowlist state is refreshed when configured and is cheaper to retire when dormant. |
| Persistent | `AccountOpCount(address)` | `86,400` | `95,040` | Quota metadata follows account activity and does not need escrow-length retention. |

These values are ledger counts, not wall-clock durations. At the common
five-second ledger close time, the active horizon is approximately 31 days,
the terminal retention horizon is approximately 28 hours, the caller horizon
is approximately 16 days, and the quota horizon is approximately 5.5 days.
Network ledger close times can change those estimates; correctness depends on
ledger counts and not on a wall-clock conversion.

### Key ownership map

| Key | Tier | Writer | Reader | Cleanup |
|---|---|---|---|---|
| `Admin` | Instance | `initialize`, `accept_admin` | admin getters and authorization | Never independently; instance lifecycle controls it. |
| `PendingAdmin` | Instance | `transfer_admin` | `get_pending_admin`, `accept_admin` | Removed on acceptance. |
| `Token` | Instance | `initialize` | token operations | Never independently. |
| `Counter` | Instance | `create_transfer` | ids and bounded query windows | Never independently. |
| `Paused` | Instance | `pause`, `unpause` | create guard | Never independently. |
| `TotalEscrowed` | Instance | create/claim/cancel/sweep | invariant checks and getter | Never independently. |
| `InitializedAt` | Instance | `initialize` | metadata getter | Never independently. |
| `LastPrivilegedCall` | Instance | admin calls | cooldown guard | Never independently. |
| `Transfer(id)` | Persistent | create and settlement | transfer queries and settlement | Only terminal records. |
| `AllowedCaller(address)` | Persistent | add/remove caller | authorization | Explicit removal by admin. |
| `AccountOpCount(address)` | Persistent | successful creates | quota guard | Natural expiry; no transfer cleanup may touch it. |

The account-operation counter is intentionally in persistent storage. Using an
`AccountOpCount` key with instance storage would couple a per-account record to
the global instance entry and would make its TTL impossible to manage
independently. It also violates the key ownership map above.

## Centralized implementation

`src/storage.rs` owns all persistent TTL decisions. `set_transfer` chooses the
active or terminal class from the transfer status, `set_caller_allowed` uses
the allowlist class, and `increment_account_op_count` uses the account quota
class. Callers do not pass arbitrary threshold/amount pairs.

The common `extend_persistent` helper uses Soroban's documented semantics:

```text
if current_ttl < threshold:
    current_ttl = extend_to
else:
    current_ttl stays unchanged
```

This makes a retry that performs the same logical write idempotent while a
record in its renewal window is restored to its class horizon. The operation
is also monotonic: a bump cannot shorten a record's remaining TTL. A terminal
transition can therefore be observed for the rest of an already-paid active
horizon, even though future terminal writes use the shorter terminal class.

Instance state has one analogous `extend_instance` helper. Since all
singleton state shares one instance entry, a successful mutating call refreshes
the instance as a unit. Read-only calls do not pay a write or TTL-refresh cost.

## Terminal cleanup contract

`cleanup_terminal_transfers(ids)` is permissionless and returns the number of
records removed. It has four deliberate rules:

- `ids.len()` must be no greater than `MAX_TERMINAL_CLEANUP` (20).
- Each id is looked up once; there is no scan from one through `Counter`.
- A record is removed only if its status is `Claimed` or `Cancelled`.
- Missing ids and repeated ids are no-ops, making retries safe.

Cleanup does not move funds, change `TotalEscrowed`, or alter the account
quota. Settlement remains the only path that changes liability. In particular,
cleanup cannot make a pending transfer disappear or bypass its expiry and
authorization rules.

Indexers should submit terminal ids after observing the settlement event. They
may submit a mixed page containing pending, terminal, and already-removed ids;
the result is deterministic and only terminal records are counted. If a
terminal record naturally expires before cleanup, the call remains successful
and reports zero for that id.

## Compatibility and migration notes

The public transfer lifecycle is unchanged: create, claim, cancel, and sweep
still use the same arguments, status values, token movements, and events. The
new cleanup method is additive. Existing clients do not need to call it;
Soroban expiry remains a fallback reclamation path.

Deployments upgrading from an earlier build should note the following:

- Existing active transfer entries keep their current TTL until a normal state
  update or a new renewal window. The policy never shortens them.
- Existing terminal entries may retain the former active horizon. This is safe
  and they can still be explicitly cleaned.
- Existing allowlist entries retain their current TTL; the next allowlist write
  uses the dedicated class.
- Account quota entries created by the older implementation are not silently
  copied between storage tiers. New writes use the declared persistent tier,
  while a deployment can apply a one-time migration if legacy quota data
  exists.

No user funds or terminal status is discarded by this change. The only new
deletion path is caller-requested cleanup guarded by the terminal-status check.

## Budget model

The normal write paths remain O(1) with respect to the number of transfers:

| Operation | Storage work added by the policy | Bound |
|---|---:|---:|
| Create | one transfer TTL bump and one account-counter TTL bump | O(1) |
| Claim/cancel/sweep | one transfer TTL bump | O(1) |
| Add caller | one allowlist TTL bump | O(1) |
| Remove caller | one persistent removal | O(1) |
| Cleanup | one lookup per requested id and at most one removal | O(k), `k <= 20` |

The cleanup endpoint intentionally does not call `get_transfers_paged`,
`total_escrowed`, or any count method. Those methods have independent query
semantics and some scan transfer ids. Keeping cleanup on direct keys avoids
turning a maintenance transaction into a transfer-counter-sized transaction.

The regression suite measures native test CPU and memory cost for a full
20-id cleanup request and verifies it stays below the default one-million CPU
instruction ceiling. Native measurements are a lower bound for WASM costs,
so the hard batch bound is the primary production guarantee; the test is a
regression alarm, not a network fee quote.

## Constants in `src/storage.rs`

- `INSTANCE_BUMP_THRESHOLD` / `INSTANCE_BUMP_AMOUNT`: `518_400` / `535_680`
- `PERSISTENT_BUMP_THRESHOLD` / `PERSISTENT_BUMP_AMOUNT`: `518_400` / `535_680`
- `CALLER_BUMP_THRESHOLD` / `CALLER_BUMP_AMOUNT`: `259_200` / `276_480`
- `ACCOUNT_OP_BUMP_THRESHOLD` / `ACCOUNT_OP_BUMP_AMOUNT`: `86_400` / `95_040`
- `TERMINAL_BUMP_THRESHOLD` / `TERMINAL_BUMP_AMOUNT`: `10_080` / `20_160`
- `MAX_TERMINAL_CLEANUP`: `20` ids per call

Every bump amount is greater than or equal to its threshold. The active
transfer class intentionally has the longest persistent horizon because it
protects escrowed funds. Shorter classes reduce rent exposure for metadata
without weakening the active-transfer guarantee.

## Automated test verification

Storage TTL behavior is validated in `src/test.rs` via automated tests:

- `test_ttl_bump_constants_configured_correctly` verifies the existing
  instance and active-transfer boundaries.
- `test_ttl_policy_classes_have_ordered_retention_windows` protects the
  relative retention order and cleanup bound.
- `test_allowlist_uses_its_dedicated_ttl_class` prevents allowlist writes from
  silently adopting the transfer horizon.
- `test_account_operation_counter_is_persistent_and_has_ttl` catches the
  instance/persistent tier regression and verifies the quota TTL.
- `test_repeated_transfer_writes_are_monotonic_and_idempotent` confirms a retry
  does not extend a healthy record beyond its policy horizon.
- `test_live_transfer_cannot_be_removed_by_cleanup` is regression coverage for
  the original active-escrow failure mode.
- `test_cleanup_removes_claimed_transfer_and_is_idempotent` and
  `test_cleanup_removes_expired_cancelled_transfer_only_after_settlement`
  verify safe terminal reclamation for both settlement paths.
- `test_cleanup_mixed_ids_counts_only_terminal_records` verifies mixed
  maintenance pages do not affect live records.
- `test_cleanup_rejects_batches_over_the_budget_bound` and
  `test_cleanup_work_is_bounded_and_budget_safe` verify hard input bounds and
  transaction-budget behavior.
- `test_active_ttl_survives_ledger_progress_within_policy` and
  `test_terminal_policy_is_shorter_than_live_policy_but_keeps_status_available`
  verify ledger progression and terminal retention.

See `docs/storage-model.md`, `docs/gas-and-fees.md`, and `src/storage.rs` for
the surrounding storage and cost model.
