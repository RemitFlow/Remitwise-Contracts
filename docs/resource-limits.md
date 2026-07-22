# Resource Limits

This note documents the **resource-limits** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the resource-limits in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Configured Operational Limits

The contract defines the following operational and resource bounds, exposed on-chain via the [`get_limits()`](entrypoint-reference.md#get_limits---configuredlimits) getter:

- **`MAX_AMOUNT` (`max_amount`)**: `1_000_000_000_000_000_000` — Upper limit on the token amount for any single transfer created.
- **`MAX_EXPIRY_WINDOW` (`max_expiry_window`)**: `31_536_000` seconds (~1 year) — Maximum allowable window between the current ledger timestamp and the transfer expiry.
- **`MAX_TOTAL_ESCROWED` (`max_total_escrowed`)**: `1_000_000_000_000_000_000` — Global ceiling on the aggregate token balance held in active escrow.
- **`MAX_PAGE_SIZE` (`max_page_size`)**: `100` — Upper bound on transfer records returned per page in `get_transfers_paged`.

