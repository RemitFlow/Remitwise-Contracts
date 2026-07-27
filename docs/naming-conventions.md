# Naming Conventions

This document defines the naming conventions used for storage keys and event
topics in the RemitFlow contract, and how they are enforced automatically.

---

## Why this matters

Both storage key variant names ([`InstanceKey`](./storage-model.md),
[`PersistentKey`](./storage-model.md)) and event topics
(see [`Event Reference`](./event-reference.md)) are encoded on-chain as a
Soroban `Symbol`. A `Symbol` only accepts the characters `a-zA-Z0-9_` and a
maximum length of 32 characters. Beyond that hard SDK limit, this contract
also enforces a specific case convention per category so that keys and
topics stay predictable, greppable, and consistent as the contract grows.

Because [storage keys are part of the on-chain interface](./storage-model.md#upgrade-notes)
(renaming a variant orphans existing data) and event topics are part of the
public interface consumed by indexers (see [Event Indexing](./event-indexing.md)),
an accidental rename or a new variant/topic that doesn't follow the
convention is a real risk, not just a style nit. It's caught automatically
by tests rather than relying on review alone.

## Conventions

| Category | Convention | Max length | Example |
| :--- | :--- | :--- | :--- |
| Storage key variants (`InstanceKey`, `PersistentKey`) | PascalCase, alphanumeric only | 32 chars | `PendingAdmin`, `AccountOpCount` |
| Event topics | snake_case, lowercase alphanumeric + `_` | 32 chars | `caller_added`, `admin_transfer_started` |

## Automated enforcement

- `src/naming_conventions.rs` defines the `is_pascal_case` / `is_snake_case`
  format validators and unit tests that assert every current `InstanceKey`
  and `PersistentKey` variant name follows the PascalCase convention within
  the 32-character `Symbol` limit. Each variant is enumerated explicitly, and
  a variant-count assertion forces a deliberate update to the test whenever a
  variant is added or removed.
- `test_event_topic_naming_convention` in `src/test.rs` asserts every event
  topic listed in [`docs/event-reference.md`](./event-reference.md) is
  snake_case and within the length limit, using the same topic list that
  [`test_event_topics_stability`](./event-reference.md#event-payload--topic-stability-verification)
  already verifies against live-emitted events — so a topic rename that
  isn't reflected in both tests fails the build.
- These are ordinary `#[test]` functions, so they run as part of the
  existing `cargo test --locked` step in [CI](../.github/workflows/ci.yml);
  no separate CI job is needed.

## Adding a new storage key or event topic

1. Pick a name that follows the convention for its category (see table
   above).
2. Add it to the corresponding registry table in `docs/storage-model.md` or
   `docs/event-reference.md`.
3. Add it to the explicit variant/topic list in `src/naming_conventions.rs`
   (storage keys) or `src/test.rs`'s `test_event_topic_naming_convention` /
   `test_event_topics_stability` (event topics), so drift is caught the next
   time `cargo test` runs.
