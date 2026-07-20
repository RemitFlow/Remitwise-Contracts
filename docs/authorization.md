# Authorization

This note documents the **authorization** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page describes the authorization in detail, covering the relevant entrypoints, storage layout, and invariants where applicable.

## Admin-Only Entrypoints
Only the configured administrator address can perform administrative operations. The contract enforces this by verifying `admin.require_auth()` for the following entrypoints:
* `pause` / `unpause`
* `add_caller` / `remove_caller`

## Admin Key Custody Model
The contract uses a single admin address configured at initialization. That address becomes the sole authority for privileged operations and is stored directly in contract instance storage. The contract does not implement admin rotation, multisig, or policy-based control inside the Wasm; all custody decisions are expected to happen off-chain.

Recommended custody practices:
* Hold the admin key in a hardware wallet or another purpose-built custody solution.
* Prefer a multisig or timelock arrangement for any administrative action that would materially affect operations.
* Keep the admin key materially segregated from day-to-day operational keys and rotate or recover it through a documented process.

Security note: compromise of the admin key can pause the contract and change the allowlist, but it cannot directly withdraw escrowed funds from the contract.

## Privileged Callers Allowlist
The contract maintains an allowlist of privileged callers who are authorized to lock funds and create new escrow transfers. 
* To create a transfer via `create_transfer`, the sender `from` address must be present on the allowlist (which is verified using `storage::is_caller_allowed`).
* The administrator can add addresses to the allowlist using `add_caller(caller)` and remove them using `remove_caller(caller)`.
* Anyone can query the allowlist status of an address using `is_caller_allowed(caller)`.

## Transfer Claiming and Cancellation
* `claim_transfer` requires authorization from the recipient address specified in the transfer.
* `cancel_transfer` requires authorization from the sender (`from`) address specified in the transfer.

