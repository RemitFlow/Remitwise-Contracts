## Deployment Funding

Deploying a Soroban contract requires the deployer account to hold sufficient XLM.

- Contract deployment: approx 5 XLM for WASM upload and instance creation
- Instance storage rent: approx 2 XLM for initial TTL allocation
- Persistent storage rent: approx 0.5 XLM per transfer record
- Transaction fee: 0.00001 XLM per invocation
- TTL extension: approx 1 XLM per year

Recommended minimum deployer balance: 20 XLM

Testnet tokens available from Stellar Friendbot at https://friendbot.stellar.org
## Multisig-Compatible Administration

The contract currently uses a single admin address. For production deployments requiring multisig security:

- The admin key can be a Stellar multisig account (e.g., 2-of-3 threshold)
- Multisig transactions require all signers to authorize via Stellar's native multisig
- No contract changes needed - Soroban respects Stellar account thresholds natively
- Future contract versions may add on-chain admin set management
## Minimum Supported SDK Version

- Soroban SDK: 21.7.6 (pinned in Cargo.toml)
- Rust toolchain: 1.81.0 (stable)
- WASM target: wasm32-unknown-unknown
```text
get_transfers_paged(1, 25)
get_transfers_paged(26, 25)
```

The contract returns at most `MAX_PAGE_SIZE` (100) records per call. A zero
limit, an empty contract, or a start id beyond the current counter returns an
empty vector. A start id of zero is treated as one.


## Admin key custody model

The contract uses a single admin address that is configured once at initialization. That address is the only authority permitted to pause or unpause the contract and to manage the privileged caller allowlist. The contract does not include an on-chain admin rotation or multisig mechanism, so key custody remains an off-chain operational responsibility.

Recommended practice is to hold the admin key in a hardware wallet or dedicated custody solution, ideally with a multisig or timelock guard for any sensitive operation. A compromised admin key can pause the contract and modify the allowlist, but it cannot directly withdraw escrowed funds from the contract.

## Build

Build the optimized WASM with the pinned toolchain:

```sh
make build
# or directly:
cargo build --target wasm32-unknown-unknown --release
```

For a detailed explanation of what makes the build deterministic and how to
verify that two builds produce the same artifact, see the
[Reproducible Builds guide](docs/reproducible-build.md).

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

Release WASM is limited to 65,536 bytes (64 KiB). Run `make check-wasm-size`
before opening a pull request; CI enforces the same limit. See the
[WASM size budget](docs/resource-limits.md) for measurement details and
guidance on keeping additions within budget.

## Deploy

Use the automated script to build, deploy, and initialize in one step:

```sh
./scripts/deploy-and-initialize.sh \
  --network  testnet \
  --source   my-key \
  --admin    GABC...XYZ \
  --token    CABC...XYZ
```

Or via `make`:

```sh
make deploy \
  NETWORK=testnet \
  SOURCE=my-key \
  ADMIN=GABC...XYZ \
  TOKEN=CABC...XYZ
```

The script builds the WASM, deploys it, and calls `initialize` in one
transaction sequence. It prints the contract ID and suggested next steps on
success. Pass `--skip-build` to reuse an already-compiled WASM.

See the [Deployment Guide](docs/deployment-guide.md) for the full options
reference, manual CLI steps, and mainnet instructions.

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