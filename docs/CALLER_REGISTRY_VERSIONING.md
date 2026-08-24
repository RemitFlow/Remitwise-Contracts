# Versioned allowed-caller registry

The allowed-caller registry controls which external addresses may create new
escrow transfers. A caller update is therefore a privileged state transition,
not a best-effort configuration write. This note defines the replay-safe API
added for integrations that need auditable and ordered registry changes.

## API

`update_caller_versioned(caller, allowed, version)` accepts an administrator-
authorized update and returns `CallerUpdateResult`.

| Result field | Meaning |
| --- | --- |
| `changed: true` | The registry membership changed and one event was emitted. |
| `duplicate: true` | The exact `(version, caller, allowed)` update was already applied. |
| `version` | The current registry version after evaluation. |

`caller_registry_version()` exposes the current version for clients preparing a
new update. Version zero is the state immediately after initialization. The
first accepted update must use version one, and every later new update must use
the next integer.

The existing `add_caller` and `remove_caller` entry points remain available for
compatibility with callers that do not yet carry a version. New integrations
should use the versioned entry point. A later breaking migration can retire the
unversioned methods after all clients and indexers have moved.

## Authorization

The contract reads the configured administrator and calls `require_auth` before
performing a versioned update. A caller address is also rejected when it is the
contract address itself. The caller being granted does not receive authority to
modify the registry; only the configured administrator does.

Authorization is checked before replay handling. This matters because a public
duplicate response must not become an oracle for an unauthorized operator to
probe registry history. An unauthorized request fails even if its tuple matches
an update that was previously accepted.

## Version and replay rules

The registry stores a monotonic instance version and a persistent marker for
each exact tuple `(version, caller, allowed)`. Evaluation follows this order:

1. load the administrator and require its authorization;
2. reject the contract's own address;
3. return a deterministic duplicate result if the exact tuple is marked;
4. require the cooldown for a new privileged transition;
5. require `version == current + 1`; and
6. commit the membership, version, marker, cooldown timestamp, and event.

An exact retry is safe even when it arrives before the cooldown expires because
it does not create a new transition. A different tuple with the same version is
not a duplicate; it is stale and returns `StaleCallerUpdate`. The version is
global to the registry, so two administrators or two integrations cannot race
independent sequences into the same state.

## State transition examples

| Current | Request | Outcome |
| ---: | --- | --- |
| 0 | `(alice, true, 1)` | Add Alice; version 1; one event. |
| 1 | same tuple | Duplicate; version remains 1; no event. |
| 1 | `(alice, false, 3)` | Stale; membership remains enabled. |
| 1 | `(alice, false, 2)` | Remove Alice; version 2; one event. |
| 2 | `(bob, true, 2)` | Stale; Bob remains disabled. |
| 2 | `(bob, true, 3)` before cooldown | Cooldown error; version remains 2. |
| 2 | `(bob, true, 3)` after cooldown | Add Bob; version 3; one event. |

The result deliberately distinguishes `changed` and `duplicate`. A client can
acknowledge an ambiguous network retry without incrementing its local event
counter or publishing a second configuration notification.

## Atomicity

An accepted update writes these pieces of state in one contract call:

- persistent caller membership;
- instance registry version;
- persistent exact-update marker;
- last privileged-call timestamp; and
- the `caller_registry_changed` event.

If a host or invariant error traps, Soroban rolls the invocation back. The
registry cannot advance without its membership write, and a replay marker cannot
remain without the corresponding accepted update. The cooldown timestamp is
also not consumed by a stale or rejected request.

The persistent marker is correctness state, not a cache. It has its own TTL
extension and must be retained as long as historical registry updates may be
retried. Archival must preserve the tuple and transaction evidence before
removing old records.

## Audit event

Every accepted versioned update emits one `caller_registry_changed` event with
the version in the topics and `(admin, caller, allowed)` in the data payload.
The version is indexed so off-chain consumers can detect gaps. Exact duplicate
retries emit no second event. The legacy `caller_added` and `caller_removed`
events remain unchanged for the compatibility methods.

Consumers should treat the versioned event stream as an append-only audit log:

- reject a new event whose version skips the expected next value;
- tolerate a duplicate delivery of the same ledger event at the indexer layer;
- verify the administrator and caller against the payload before display; and
- use the membership query as current state, not as a substitute for history.

The event intentionally carries addresses and a boolean only. It does not carry
free-form metadata or secrets, keeping indexing bounded and minimizing accidental
disclosure in monitoring systems.

## Removal and mutation gating

`create_transfer` checks `is_caller_allowed` before validating and moving funds.
After a successful versioned removal, the removed address fails that check for
new mutations. Existing escrow transfers are not retroactively canceled: their
sender and recipient lifecycle permissions remain governed by the transfer
state. This separation avoids confiscating already locked funds while preventing
new escrow creation by a removed integration.

Clients should wait for the removal transaction to be finalized before sending
new transfer requests from the removed address. A request already executing in
the same transaction is governed by Soroban atomicity; there is no partial
mid-call registry observation.

## Cooldown interaction

Privileged calls use the existing five-minute cooldown. The first call at ledger
timestamp zero is compatible with the historical contract behavior; deployments
should use a nonzero ledger timestamp for operational sequencing. A duplicate
does not consume cooldown because it makes no state change. A stale request also
does not consume cooldown. Only an accepted membership transition advances the
privileged-call timestamp.

This ordering allows clients to retry an accepted update during a network
timeout while still protecting the registry from rapid distinct changes. The
client should not alter the tuple when retrying an ambiguous request.

## Client workflow

1. Read `caller_registry_version`.
2. Select `next = current + 1`.
3. Build the desired caller and membership state.
4. Have the configured administrator authorize the call.
5. Submit `update_caller_versioned`.
6. On an ambiguous response, retry the exact same tuple.
7. Treat `duplicate: true` as reconciliation success.
8. On `StaleCallerUpdate`, reread the version and reconcile governance changes.
9. After removal, stop submitting new mutations from that caller.

Do not pre-sign a large batch of future versions. A different accepted update
will make those requests stale, and a compromised pre-signed sequence would be
difficult to revoke without rotating the administrator.

## Failure and recovery

The method can return `NotInitialized`, `InvalidAddress`,
`CooldownNotElapsed`, `StaleCallerUpdate`, or
`CallerUpdateVersionOverflow`. Each is a non-success state and should be
recorded by the client without advancing its local version.

If a request returns a cooldown error, wait until the ledger timestamp satisfies
the existing cooldown and retry the same next version. If it returns stale,
never blindly retry with the same version; another update has already advanced
the registry. If the administrator key is unavailable, use the contract's
existing two-step admin transfer process before attempting recovery.

## Compatibility and migration

The new instance key and persistent marker variant are additive. Existing admin,
token, transfer, and allowlist storage keys retain their encoding. Existing
readers of `is_caller_allowed` continue to work, and a deployment can adopt the
new method without migrating current membership entries.

Migration steps:

1. Deploy the contract version containing the versioned API.
2. Record the current version as zero for a new deployment or the documented
   migration baseline for an existing deployment.
3. Move one integration at a time to versioned updates.
4. Confirm one event per accepted change in the indexer.
5. Verify removal blocks a new `create_transfer` call.
6. Retire unversioned configuration calls in application code.

Rollback must preserve the version and replay-marker state. Removing the marker
would allow an old accepted update to be replayed as a new configuration action
after re-enablement. If a rollback cannot preserve these keys, disable versioned
updates until a state migration has been reviewed.

## Test evidence

The regression suite covers:

- version-zero baseline and version-one acceptance;
- exact duplicate result with unchanged version;
- one-event audit behavior;
- removal and new-mutation blocking;
- stale version rejection without membership mutation;
- cooldown rejection without consuming a version;
- ordered updates for multiple callers; and
- existing allowlist, transfer, authorization, and invariant coverage.

The full suite runs 134 passing tests with 4 pre-existing ignored fixture/event
tests. No test is disabled or deleted by this change.

## Review checklist

- [ ] All new integrations use the versioned method.
- [ ] Administrator authorization is verified before duplicate disclosure.
- [ ] Exact duplicates return without a second event.
- [ ] Stale updates leave membership and version unchanged.
- [ ] Accepted removal blocks new caller mutations.
- [ ] Cooldown failures do not consume a version.
- [ ] Replay markers have persistent TTL handling.
- [ ] Rollback preserves registry version and markers.
- [ ] Indexers alert on version gaps.
- [ ] Full CI passes without skipped checks.
