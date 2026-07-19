# Authorization

This note documents the **authorization** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page describes the authorization in detail, covering the relevant entrypoints, storage layout, and invariants where applicable.

## Admin-Only Entrypoints
Only the configured administrator address can perform administrative operations. The contract enforces this by verifying `admin.require_auth()` for the following entrypoints:
* `pause` / `unpause`
* `add_caller` / `remove_caller`

## Privileged Callers Allowlist
The contract maintains an allowlist of privileged callers who are authorized to lock funds and create new escrow transfers. 
* To create a transfer via `create_transfer`, the sender `from` address must be present on the allowlist (which is verified using `storage::is_caller_allowed`).
* The administrator can add addresses to the allowlist using `add_caller(caller)` and remove them using `remove_caller(caller)`.
* Anyone can query the allowlist status of an address using `is_caller_allowed(caller)`.

## Transfer Claiming and Cancellation
* `claim_transfer` requires authorization from the recipient address specified in the transfer.
* `cancel_transfer` requires authorization from the sender (`from`) address specified in the transfer.

