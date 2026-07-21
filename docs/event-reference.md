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

## Event Payload Verification
All events emitted by the contract are tested for topic alignment and payload structural integrity inside the automated test suite under the `test_event_payload_contents` unit test case.
