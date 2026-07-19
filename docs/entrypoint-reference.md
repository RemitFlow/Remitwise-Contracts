# Entrypoint Reference

This note documents the **entrypoint-reference** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page describes the entrypoint reference in detail.

## Privileged Callers Allowlist Management

### `add_caller(caller: Address) -> Result<(), Error>`
Adds an address to the privileged callers allowlist.
* **Authorization**: Admin
* **Events**: Emits `caller_added` event with the caller's address.
* **Errors**: `NotInitialized` if the contract is not initialized, `AlreadyInitialized` or others from invalid admin authentication.

### `remove_caller(caller: Address) -> Result<(), Error>`
Removes an address from the privileged callers allowlist.
* **Authorization**: Admin
* **Events**: Emits `caller_removed` event with the caller's address.
* **Errors**: `NotInitialized` if the contract is not initialized.

### `is_caller_allowed(caller: Address) -> bool`
Queries whether the given address is authorized on the privileged callers allowlist.
* **Authorization**: None (Public view)

## Transfer Queries

### `get_transfers_paged(start_id: u64, limit: u32) -> Vec<Transfer>`

Returns a bounded page of transfer records ordered by ascending transfer id.

* **Authorization**: None (public view)
* **Cursor**: `start_id` is inclusive; `0` is treated as transfer id `1`
* **Page size**: Returns at most `min(limit, MAX_PAGE_SIZE)`, where
  `MAX_PAGE_SIZE` is 100
* **Empty pages**: Returns an empty vector when `limit` is zero, no transfers
  exist, or `start_id` is beyond the current transfer counter
* **Next page**: If a full page is returned, pass the last returned transfer
  id plus one as the next `start_id`

