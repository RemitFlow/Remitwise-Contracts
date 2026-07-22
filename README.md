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