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
| `create_transfer(from, recipient, amount, expiry) -> u64` | Lock funds in escrow and return the transfer id. |
| `claim_transfer(id, recipient)` | Recipient claims a pending, unexpired transfer. |
| `cancel_transfer(id, from)` | Sender reclaims a pending transfer after expiry. |
| `get_transfer(id) -> Transfer` | Read a stored transfer record. |
| `get_admin() -> Address` | Return the configured admin. |
| `get_token() -> Address` | Return the configured token. |
| `counter() -> u64` | Return the number of transfers created. |
