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

## Build

Build the optimized WASM with the pinned toolchain:

```sh
make build
# or directly:
cargo build --target wasm32-unknown-unknown --release
```

Run the test suite:

```sh
make test
# or directly:
cargo test
```

The compiled artifact is written to
`target/wasm32-unknown-unknown/release/remitflow_contract.wasm`.

## Deploy

Deploy the WASM and initialize it using the Stellar CLI:

```sh
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/remitflow_contract.wasm \
  --source deployer \
  --network testnet

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS>
```

## Transfer lifecycle

Each transfer moves through the following states:

- `Pending` — created and funded, awaiting action.
- `Claimed` — recipient withdrew the funds before expiry (terminal).
- `Cancelled` — sender reclaimed the funds after expiry (terminal).

Only `Pending` transfers can be claimed or cancelled. Claims must happen on or
before the expiry timestamp; cancellations are only allowed strictly after it.

## License

Licensed under the MIT License.
