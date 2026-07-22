# Entrypoint Reference

This note documents the **entrypoint-reference** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page describes the entrypoint reference in detail.

## Batch Operations

### `batch_operations(operations: Vec<BatchOperation>) -> Result<Vec<BatchOperationResult>, Error>`

Executes an ordered collection of `Create`, `Claim`, and `Cancel` operations in
a single invocation. Each operation uses the same validation and authorization
rules as its corresponding standalone entrypoint.

The batch is atomic. When any operation returns an error, Soroban rolls back all
earlier operations in that batch, including token movements, storage changes,
and emitted events. An empty batch succeeds and returns an empty result vector.

Successful results preserve input order. `Create` returns `Created(id)`;
`Claim` returns `Claimed`; and `Cancel` returns `Cancelled`.

## Admin Management

### `transfer_admin(new_admin: Address) -> Result<(), Error>`
Initiates a two-step admin ownership transfer by nominating a successor.
* **Authorization**: Current admin (`admin.require_auth()`)
* **Effect**: Stores `new_admin` as the pending admin. The current admin retains
  all privileges until `accept_admin` is called. A subsequent call replaces any
  existing pending admin.
* **Events**: Emits `admin_transfer_started` with `(current_admin, new_admin)`.
* **Errors**: `NotInitialized` if the contract is not initialized.

### `accept_admin() -> Result<(), Error>`
Completes a two-step admin ownership transfer.
* **Authorization**: Pending admin (`pending.require_auth()`)
* **Effect**: Overwrites the admin slot with the pending admin address and clears
  the pending-admin slot. The caller must be the exact address nominated by the
  most recent `transfer_admin` call.
* **Events**: Emits `admin_transfer_completed` with `(old_admin, new_admin)`.
* **Errors**: `NoPendingAdmin` if no transfer has been initiated; `NotInitialized`
  if the contract is not initialized.

### `get_pending_admin() -> Option<Address>`
Returns the currently nominated pending admin address.
* **Authorization**: None (public view)
* Returns `None` when no transfer is in progress.

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

