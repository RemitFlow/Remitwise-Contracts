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

