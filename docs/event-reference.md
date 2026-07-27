# Event Reference

This document describes all events emitted by the RemitFlow smart contract, detailing their topics and data payloads to facilitate event indexing and monitoring.

## Event Schema Table

| Event Name | Topics | Data Payload | Trigger Condition |
| :--- | :--- | :--- | :--- |
| `init` | `("init",)` | `(admin: Address, token: Address)` | Contract initialization. |
| `caller_added` | `("caller_added",)` | `(caller: Address)` | Caller added to the allowlist. |
| `caller_removed` | `("caller_removed",)` | `(caller: Address)` | Caller removed from the allowlist. |
| `paused` | `("paused",)` | `(admin: Address)` | Contract paused by admin. |
| `unpaused` | `("unpaused",)` | `(admin: Address)` | Contract unpaused by admin. |
| `created` | `("created", id: u64)` | `(from: Address, recipient: Address, amount: i128, expiry: u64)` | A new transfer is created and funds escrowed. |
| `claimed` | `("claimed", id: u64)` | `(recipient: Address, amount: i128)` | Recipient claims escrowed transfer. |
| `cancelled` | `("cancelled", id: u64)` | `(from: Address, amount: i128)` | Sender cancels and receives a refund for an expired transfer. |
| `admin_transfer_started` | `("admin_transfer_started",)` | `(current_admin: Address, pending_admin: Address)` | Admin initiates ownership transfer. |
| `admin_transfer_completed` | `("admin_transfer_completed",)` | `(old_admin: Address, new_admin: Address)` | Pending admin accepts ownership transfer. |

## Savings Goal Events

Unlike the events above, which publish positional tuples, savings goal
events publish an explicit `#[contracttype]` payload struct as their data —
every field is named, giving indexers a self-describing, versionable schema
instead of a positional tuple they must decode by convention. `amount` on
`goal_deposited`/`goal_withdrawn` is the delta applied by that call;
`new_total` is the goal's resulting `current_amount`.

| Event Name | Topics | Data Payload | Trigger Condition |
| :--- | :--- | :--- | :--- |
| `goal_created` | `("goal_created", goal_id: u64)` | `GoalCreatedEvent { goal_id: u64, owner: Address, target_amount: i128, deadline: u64, timestamp: u64 }` | A new savings goal is created. |
| `goal_deposited` | `("goal_deposited", goal_id: u64)` | `GoalDepositedEvent { goal_id: u64, owner: Address, amount: i128, new_total: i128, timestamp: u64 }` | Owner deposits into a goal. |
| `goal_withdrawn` | `("goal_withdrawn", goal_id: u64)` | `GoalWithdrawnEvent { goal_id: u64, owner: Address, amount: i128, new_total: i128, timestamp: u64 }` | Owner withdraws from a goal. |
| `goal_completed` | `("goal_completed", goal_id: u64)` | `GoalCompletedEvent { goal_id: u64, owner: Address, final_amount: i128, timestamp: u64 }` | A deposit brings `current_amount` to or past `target_amount`. |
| `goal_cancelled` | `("goal_cancelled", goal_id: u64)` | `GoalCancelledEvent { goal_id: u64, owner: Address, refunded_amount: i128, timestamp: u64 }` | Owner cancels a goal; any balance is refunded. |

## Event Payload & Topic Stability Verification
All events emitted by the contract are tested for topic alignment, payload structural integrity, and topic symbol stability inside the automated test suite under the `test_event_payload_contents` and `test_event_topics_stability` unit test cases.

Savings goal event topics and structured payloads are locked down in `test_goal_event_schema_stability`, which exercises every goal lifecycle transition (create, deposit, withdraw, cancel, and deposit-to-completion) against live-emitted events.
