# Event Reference

This document describes all events emitted by the RemitFlow smart contract, detailing their topics and versioned data payloads to facilitate event indexing and monitoring. The complete decoder and compatibility policy is in [VERSIONED_EVENT_SCHEMAS.md](VERSIONED_EVENT_SCHEMAS.md).

## Event Schema Table

| Event Name | Topics | Data Payload | Trigger Condition |
| :--- | :--- | :--- | :--- |
| `init` | `("init",)` | `InitEvent` (versioned) | Contract initialization. |
| `caller_added` | `("caller_added",)` | `ActorEvent` (versioned) | Caller added to the allowlist. |
| `caller_removed` | `("caller_removed",)` | `ActorEvent` (versioned) | Caller removed from the allowlist. |
| `paused` | `("paused",)` | `ActorEvent` (versioned) | Contract paused by admin. |
| `unpaused` | `("unpaused",)` | `ActorEvent` (versioned) | Contract unpaused by admin. |
| `created` | `("created", id: u64)` | `CreatedEvent` (versioned; amount in token base units; expiry in ledger seconds) | A new transfer is created and funds escrowed. |
| `claimed` | `("claimed", id: u64)` | `ClaimedEvent` (versioned; amount in token base units) | Recipient claims escrowed transfer. |
| `cancelled` | `("cancelled", id: u64)` | `CancelledEvent` (versioned; amount in token base units) | Sender cancels and receives a refund for an expired transfer. |
| `admin_transfer_started` | `("admin_transfer_started",)` | `AdminTransferEvent` (versioned) | Admin initiates ownership transfer. |
| `admin_transfer_completed` | `("admin_transfer_completed",)` | `AdminTransferEvent` (versioned) | Pending admin accepts ownership transfer. |

## Event Payload & Topic Stability Verification
All events emitted by the contract are tested for topic alignment, payload structural integrity, schema version, explicit units, and topic symbol stability inside `event_schema_tests`. Legacy lifecycle tests remain available for compatibility checks.
