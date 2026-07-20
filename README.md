# RemitFlow Contract

RemitFlow is a cross-border remittance escrow smart contract for the
[Soroban](https://soroban.stellar.org) platform on Stellar.

A sender locks token funds in escrow for a recipient with an expiry deadline.
The recipient can claim the funds before expiry; if they do not, the sender can
cancel the transfer and reclaim the funds after the deadline passes.

## Entrypoints

| Function | Description |
| --- | --- |
| `initialize(admin, token)` | Configure the admin and token; callable once. |
| `create_transfer(from, recipient, amount, expiry) -> u64` | Lock funds in escrow and return the transfer id. Caller `from` must be allowlisted. |
| `batch_operations(operations) -> Vec<BatchOperationResult>` | Atomically execute an ordered batch of create, claim, and cancel operations. |
| `claim_transfer(id, recipient)` | Recipient claims a pending, unexpired transfer. |
| `cancel_transfer(id, from)` | Sender reclaims a pending transfer after expiry. |
| `pause()` | Admin pauses creation of new transfers. |
| `unpause()` | Admin re-enables creation of new transfers. |
| `add_caller(caller)` | Add a caller to the allowlist of privileged callers (admin-only). |
| `remove_caller(caller)` | Remove a caller from the allowlist of privileged callers (admin-only). |
| `is_caller_allowed(caller) -> bool` | Check whether a caller is on the privileged callers allowlist. |
| `get_transfer(id) -> Transfer` | Read a stored transfer record. |
| `get_transfers_paged(start_id, limit) -> Vec<Transfer>` | Read up to 100 transfers beginning at the inclusive transfer id. |
| `get_status(id) -> Status` | Read just a transfer's lifecycle status. |
| `is_expired(id) -> bool` | Check whether a transfer has passed its expiry. |
| `is_paused() -> bool` | Report whether the contract is paused. |
| `transfer_exists(id) -> bool` | Check whether a transfer id has been recorded. |
| `count_by_status(status) -> u64` | Count created transfers with a given status. |
| `count_for_sender(from) -> u64` | Count transfers funded by an address. |
| `count_for_recipient(recipient) -> u64` | Count transfers targeting an address. |
| `total_escrowed() -> i128` | Sum the amounts of all pending transfers using saturating arithmetic so the aggregate clamps instead of overflowing. |
| `get_admin() -> Address` | Return the configured admin. |
| `get_token() -> Address` | Return the configured token. |
| `counter() -> u64` | Return the number of transfers created. |

### Paginating transfers

Call `get_transfers_paged(start_id, limit)` to read records without loading the
entire transfer history. Transfer ids begin at `1`, and `start_id` is
inclusive, so the next page begins at one more than the final id returned:

```text
get_transfers_paged(1, 25)
get_transfers_paged(26, 25)