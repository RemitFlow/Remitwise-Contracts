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
| `get_transfers_paged(start_id, limit) -> Vec<Transfer>` | Read a batch of transfers. |
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

Reusable checked arithmetic, saturating aggregate helpers, and basis-point fee
calculations are provided by `src/math.rs`. See the
[math module guide](docs/math-module.md) for behavior and usage rules.

Generate an HTML coverage report:

```sh
cargo install cargo-llvm-cov --locked
make coverage
```

Open `target/llvm-cov/html/index.html` in a browser to inspect the report. See
the [testing guide](docs/testing-guide.md#coverage) for LCOV output and CI
details.

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

## Aggregate behaviour

Aggregate helpers now use saturating fallbacks so counters and tallies avoid overflowing in extreme cases. This keeps transfer counts and escrow totals bounded even when many transfers are recorded.
## Global escrow cap

The contract now enforces a global cap on the total escrowed amount so the system does not accumulate an unbounded balance. Creating a transfer that would exceed this cap returns an explicit error.

## Transfer lifecycle

Each transfer moves through the following states:

- `Pending` — created and funded, awaiting action.
- `Claimed` — recipient withdrew the funds before expiry (terminal).
- `Cancelled` — sender reclaimed the funds after expiry (terminal).

Only `Pending` transfers can be claimed or cancelled. Claims must happen on or
before the expiry timestamp; cancellations are only allowed strictly after it.

## Batch operations

`batch_operations` accepts an ordered `Vec<BatchOperation>` containing
`Create`, `Claim`, and `Cancel` variants. It returns one result per operation,
including the id assigned to each created transfer. The call is atomic: if any
operation fails validation or authorization, the entire batch is rolled back,
including earlier token transfers, state changes, and events.

## License

Licensed under the MIT License.

## Resource Costs

### CPU Instructions

| Operation | CPU (approx) | Notes |
|-----------|-------------|-------|
| initialize | ~2M | One-time setup |
| create_transfer | ~8M | Token transfer + storage write |
| claim_transfer | ~7M | Token transfer + storage update |
| cancel_transfer | ~7M | Token transfer + storage update |
| pause / unpause | ~1M | Simple flag toggle |

### Storage Footprint

| Item | Persistent | Instance | TTL |
|------|-----------|----------|-----|
| Transfer record | 1 per transfer | - | Extended on write |
| Admin + Token | - | 2 | Extended on write |

### Gas Optimization Tips

- Use get_transfers_paged instead of multiple get_transfer calls
- Archive old transfers off-chain to free storage
- Keep page limits at 50 or below for predictable gas
- Monitor TTL to prevent garbage collection of active entries

## Upgrade Authority Model

The RemitFlow contract follows a single-admin authority model for upgrades.

### Authority

- The admin address set at initialization is the sole upgrade authority
- Only the admin can pause/unpause the contract
- Admin key compromise would allow an attacker to pause the contract indefinitely

### Upgrade Process

1. Deploy new WASM with stellar contract deploy
2. Invoke migrate function (if added in future) or redeploy
3. Existing transfer state is stored per-contract-instance

### Security Considerations

- Use a hardware wallet or multisig for the admin key
- Consider a timelock for sensitive admin operations
- The admin cannot steal escrowed funds (only pause new transfers)
- Future versions may add admin transfer or multisig support
