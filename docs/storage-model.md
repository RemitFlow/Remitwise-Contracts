# Storage Model

This document describes the storage architecture of the RemitFlow contract: how keys are structured, why they are split into two enums, how Soroban serialises them on-chain, and what the collision-safety guarantees are.

---

## Storage Tiers

RemitFlow uses two Soroban storage tiers, each with its own TTL lifecycle.

| Tier | Enum | Use |
|---|---|---|
| Instance | `InstanceKey` | Singleton config values shared TTL with the contract instance |
| Persistent | `PersistentKey` | Per-record and per-address values with individual TTLs |

### Why two enums?

A single `DataKey` enum covering both tiers is a common starting point, but it means that nothing at the type level prevents passing a `Transfer(id)` key to `env.storage().instance()`, or passing an `Admin` key to `env.storage().persistent()`. Such a mis-routed write would silently create an orphaned entry in the wrong tier and potentially be invisible to the TTL bump logic.

Splitting into `InstanceKey` and `PersistentKey` makes every helper function accept only the correct type, turning a potential silent runtime bug into a **compile error**.

---

## Key Encoding (Collision Safety)

Soroban serialises `#[contracttype]` enum keys as an XDR `ScVec`:

```
[ Symbol("VariantName"), payload ]
```

The **variant name string** is always part of the serialised key. This means:

- `InstanceKey::Admin` → `["Admin"]`
- `InstanceKey::PendingAdmin` → `["PendingAdmin"]`
- `PersistentKey::Transfer(1)` → `["Transfer", 1u64]`
- `PersistentKey::Transfer(2)` → `["Transfer", 2u64]`
- `PersistentKey::AllowedCaller(addr)` → `["AllowedCaller", addr]`

**No two distinct variants can ever produce the same on-chain key**, even if their payload bytes happen to be identical, because the name string differs. There is therefore no discriminant-collision risk from the SDK's own encoding scheme.

### Cross-tier isolation

Instance and persistent stores are separate namespaces at the Soroban host level. A key written to one tier is invisible to the other regardless of its serialised form. The enum split enforces this at the Rust type level.

---

## Key Registry

### `InstanceKey` — instance storage

| Variant | Value type | Description |
|---|---|---|
| `Admin` | `Address` | Current administrator address |
| `PendingAdmin` | `Address` | Nominated successor (only present during a two-step transfer) |
| `Token` | `Address` | Soroban token contract used for all escrow movements |
| `Counter` | `u64` | Monotonically increasing id issued to the next transfer |
| `Paused` | `bool` | When `true`, `create_transfer` is blocked |

### `PersistentKey` — persistent storage

| Variant | Payload | Value type | Description |
|---|---|---|---|
| `Transfer(u64)` | Transfer id | `Transfer` struct | Full record for a single escrow transfer |
| `AllowedCaller(Address)` | Caller address | `bool` | Allowlist membership flag |

---

## TTL Bump Strategy

### Instance storage

`extend_instance` is called at the end of every mutating entrypoint, extending the instance TTL to `INSTANCE_BUMP_AMOUNT` ledgers (≈ 535 680) whenever it falls below the `INSTANCE_BUMP_THRESHOLD` (≈ 518 400).

### Persistent storage

Each persistent entry is extended individually at the point of write:

- `set_transfer` extends the `Transfer(id)` entry to `PERSISTENT_BUMP_AMOUNT` ledgers.
- `set_caller_allowed` extends the `AllowedCaller(addr)` entry on insertion.
- Entries removed via `set_caller_allowed(…, false)` are deleted immediately.

Both thresholds and amounts are the same for instance and persistent storage in the current configuration (`518_400` / `535_680`), but are declared as separate constants so they can be tuned independently.

---

## Invariants

1. **No two `Transfer` records share an id.** The monotonic counter in `InstanceKey::Counter` is incremented atomically before every `set_transfer` call.
2. **`AllowedCaller` entries are independent per address.** Adding or removing one address has no effect on any other address's entry.
3. **`Transfer` and `AllowedCaller` keys are disjoint namespaces.** Their variant name strings differ, so no payload value can produce a collision.
4. **Instance and persistent entries are tier-isolated.** The split enum design enforces this at compile time.

---

## Upgrade Notes

> [!WARNING]
> Soroban encodes keys by **variant name string**. Renaming any variant in `InstanceKey` or `PersistentKey` changes its on-chain key and permanently orphans all previously stored data under the old name. Treat all variant names as part of the public storage interface.

Adding a new variant is always safe. Removing an unused variant is safe once all on-chain data under that key has been cleared or migrated.
