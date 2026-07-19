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

