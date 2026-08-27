# Versioned lifecycle event schemas

This document defines the event contract implemented for issue #194. RemitFlow
events are consumed by indexers, accounting systems, reconciliation workers,
and user interfaces. A topic name alone is not enough to make an event safe to
parse: the payload must identify its schema version, actors, identifiers,
units, and lifecycle outcome.

## Design goals

The event design has five goals:

1. Every emitted lifecycle payload declares a schema version.
2. Transfer identity is available in both the topic and decoded payload.
3. Token amounts and ledger timestamps state their units explicitly.
4. Existing event topic names and topic arity remain stable for indexers.
5. Fixture tests fail when a payload is missing a required field or changes its
   version without an intentional migration.

## Common metadata

Every payload contains `metadata`:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | `u32` | Version of this payload shape |
| `amount_unit` | `String` | `token_base_units` when amounts are present |
| `timestamp_unit` | `String` | `ledger_seconds` when timestamps are present |

An empty unit is deliberate for events that do not contain that kind of
quantity. The version is always present, including administrative events.

The current version is `1`. The version is part of the data payload rather
than a topic suffix so existing topic subscriptions continue to work. A
consumer must inspect `metadata.schema_version` before decoding a payload.

## Transfer lifecycle

### `created`

Topics remain `("created", transfer_id)`.

The payload is `CreatedEvent`:

| Field | Type | Description |
| --- | --- | --- |
| `metadata` | `EventMetadata` | Version and units |
| `transfer_id` | `u64` | Stable transfer identity |
| `from` | `Address` | Funding actor |
| `recipient` | `Address` | Claim authority |
| `amount` | `i128` | Escrow amount in token base units |
| `expiry` | `u64` | Ledger timestamp in ledger seconds |

This event is emitted only after the token transfer and escrow record have
been successfully written. A failed create call produces no successful
`created` event.

### `claimed`

Topics remain `("claimed", transfer_id)`.

The payload is `ClaimedEvent` with `metadata`, `transfer_id`, `recipient`, and
`amount`. The amount is in token base units. The event represents a completed
payment to the recipient, not merely an attempted claim.

### `cancelled`

Topics remain `("cancelled", transfer_id)`.

The payload is `CancelledEvent` with `metadata`, `transfer_id`, `from`, and
`amount`. It represents a refund after an eligible cancellation or expiry.
Indexers should use the payload `transfer_id` when reconstructing state and
should not infer the amount's decimal precision from a display token symbol.

## Administrative lifecycle

The following events use the common `ActorEvent` payload:

| Event | Topic | Actor field |
| --- | --- | --- |
| Initialization | `("init",)` | `admin` and `token` in `InitEvent` |
| Allowlist add | `("caller_added",)` | `actor` |
| Allowlist remove | `("caller_removed",)` | `actor` |
| Pause | `("paused",)` | `actor` |
| Unpause | `("unpaused",)` | `actor` |

`init` uses `InitEvent` because it records both the administrator and token
contract. All other single-actor administrative actions use `ActorEvent`.
Administrative events include `timestamp_unit = ledger_seconds` to establish
the time basis for consumers that correlate them with ledger activity, even
though the current payload does not repeat the timestamp as a field.

Two-step administrator rotation uses `AdminTransferEvent`:

| Field | Type | Description |
| --- | --- | --- |
| `metadata` | `EventMetadata` | Version metadata |
| `old_admin` | `Address` | Current administrator before the transition |
| `new_admin` | `Address` | Nominee or newly accepted administrator |

`admin_transfer_started` and `admin_transfer_completed` retain their existing
single-topic shape. Their payload field names distinguish the old and new
roles and prevent consumers from having to infer direction from tuple order.

## Indexer decoding algorithm

An indexer should process an event as follows:

1. Match the event contract address and topic symbol.
2. Validate the expected topic arity.
3. Decode the payload into the schema associated with the topic.
4. Read `metadata.schema_version`.
5. Dispatch to the version-specific decoder.
6. Validate required addresses, identifiers, and units.
7. Apply the lifecycle transition using the transaction's ledger position.
8. Persist the raw event and decoded representation for replay and audit.

An unknown schema version should be retained as an undecoded event and placed
in a review queue. It must not be silently interpreted as version `1`.

## Compatibility policy

Adding an optional field requires a new schema version when the Soroban value
shape changes. Reordering struct fields, changing a numeric type, changing a
unit string, removing an existing field, or changing topic arity is breaking.

The compatibility rules are:

- existing topic names remain stable;
- existing topic positions retain their meaning;
- the payload version is monotonically increased for breaking changes;
- a new decoder is added before a new version is emitted;
- fixtures for the prior version remain in the test suite;
- migration code documents how old events are replayed.

The contract currently emits version `1`. The structs are explicit so a future
version can be added without returning to anonymous tuples whose field meaning
is easy to lose across SDKs.

## Units and numeric safety

`amount` fields are `i128` token base units. Consumers must not convert them to
JavaScript floating point or assume a fixed number of decimal places. Decimal
display conversion belongs to a token metadata layer outside this event.

`expiry` is a Soroban ledger timestamp measured in seconds. It is not a Unix
millisecond value and must not be multiplied or divided during contract event
decoding. The ledger timestamp used for lifecycle decisions is authoritative.

Addresses are decoded as the chain's address type. Consumers should preserve
the canonical encoded form and should not treat a contract address as a user
account without checking its address type.

## Failure-mode behavior

Events are emitted after the state-changing operation has passed its guards.
Authentication failures, invalid amounts, invalid addresses, expiry failures,
paused-contract rejection, and supply-invariant failures do not produce a
successful lifecycle event.

For a batch transaction, each successful operation emits its own event in
operation order. If the batch invocation reverts, Soroban transaction
semantics discard the state changes and associated events. An indexer should
process the committed transaction result rather than treating an attempted
simulation as final state.

## Fixture and parser requirements

The `event_schema_tests` module verifies:

- every event carries version `1` metadata;
- create includes transfer id, actors, amount, expiry, and units;
- claim and cancellation retain transfer identity and amount;
- administrative events share the actor schema;
- admin rotation distinguishes old and new administrators;
- topic names and arity remain stable while payloads are structured.

The tests decode the exact Soroban event value emitted by the event helper
inside a contract context. This catches a mismatch between a Rust struct's
shape and the serialized value that an indexer receives.

## Operational rollout

Consumers deploying alongside the contract should:

1. Add version-aware decoders before consuming the new deployment.
2. Continue subscribing to the existing topic names.
3. Record unknown versions without dropping the raw event.
4. Compare decoded transfer totals with on-chain transfer getters.
5. Alert on a unit mismatch or unknown schema version.
6. Reprocess retained raw events after decoder fixes.

During rollout, the topic names do not change, so subscriptions do not need to
be recreated. The payload decoder is the compatibility boundary.

## Security considerations

Event data is an observable record, not an authorization proof. Indexers must
not grant permissions solely because an address appears in an event. They must
verify the committed transaction and current contract state. Similarly, an
event's amount must not be trusted to override the stored transfer record when
reconciling balances.

Version metadata prevents a decoder from accidentally treating a newer field
layout as an older one. Explicit units prevent a base-unit amount from being
mistaken for a decimal display amount, which could cause a material accounting
error.

## Validation commands

Run the event schema tests with:

```text
cargo +1.88.0 test --lib event_schema_tests
```

Run the complete contract unit suite with:

```text
cargo +1.88.0 test --lib
```

Run formatting verification with:

```text
cargo +1.88.0 fmt --all -- --check
```

The unit suite also retains the pre-existing lifecycle, authorization,
balance, TTL, and invariant tests.
