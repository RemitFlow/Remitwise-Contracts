# Data Types

This note documents the **data-types** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the data-types in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## ConfiguredLimits

`ConfiguredLimits` represents the static operational bounds and limits configured for the contract.

| Field | Type | Description |
| --- | --- | --- |
| `max_amount` | `i128` | Largest token amount accepted for a single escrowed transfer (`1_000_000_000_000_000_000`). |
| `max_expiry_window` | `u64` | Maximum allowed distance, in seconds, between current timestamp and transfer expiry (`31_536_000`, ~1 year). |
| `max_total_escrowed` | `i128` | Global cap on the total escrowed amount (`1_000_000_000_000_000_000`). |
| `max_page_size` | `u32` | Maximum number of records returned by a paginated transfer query (`100`). |

## SavingsGoal

`SavingsGoal` represents a single owner's progress toward a target token amount by a deadline. See [Entrypoint Reference](./entrypoint-reference.md#savings-goals) for the entrypoints that create and mutate it, and [Event Reference](./event-reference.md#savings-goal-events) for its lifecycle events.

| Field | Type | Description |
| --- | --- | --- |
| `id` | `u64` | Unique sequential identifier for this goal. |
| `owner` | `Address` | Address that created and controls this goal. |
| `target_amount` | `i128` | Token amount the owner is saving toward. |
| `current_amount` | `i128` | Token amount currently deposited toward the target. |
| `deadline` | `u64` | Ledger timestamp by which the owner intends to reach the target. |
| `created_at` | `u64` | Ledger timestamp at which the goal was created. |
| `status` | `GoalStatus` | Current lifecycle status of the goal. |

## GoalStatus

`GoalStatus` is the lifecycle status of a `SavingsGoal`.

| Variant | Value | Description |
| --- | --- | --- |
| `Active` | `0` | The goal is open and accepting deposits/withdrawals. |
| `Completed` | `1` | The goal reached or exceeded its target amount. |
| `Cancelled` | `2` | The owner cancelled the goal; any balance has been refunded. |

