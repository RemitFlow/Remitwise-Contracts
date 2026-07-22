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

