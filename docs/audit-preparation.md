# Audit Preparation

This page documents the information auditors need to review the RemitFlow
contract.

remitflow-contract is a Soroban smart contract on the Stellar network.

## Build verification

Auditors must confirm that the deployed WASM matches the audited source. The
[Reproducible Builds guide](reproducible-build.md) covers:

- Pinned Rust toolchain (`1.90.0`) — every build uses the same compiler.
- Locked dependencies (`Cargo.lock` + `--locked`) — no version drift.
- Release profile settings that eliminate non-determinism (`codegen-units = 1`,
  `lto = true`, `strip = "symbols"`, `panic = "abort"`).
- Step-by-step instructions for producing and verifying the WASM hash.
- The `verify-wasm-hash.sh` script for comparing local and on-chain hashes.

## Scope

See the README and the sources under `src/` for the authoritative
implementation. Key modules:

| Module | Path | Purpose |
|---|---|---|
| Contract | `src/lib.rs` | Entrypoints and business logic |
| Math | `src/math.rs` | Checked/saturating arithmetic, fee calculation |
| Storage | `src/storage.rs` | Persistent and instance storage helpers |
| Error | `src/error.rs` | Error types and numeric codes |
| Events | `src/events.rs` | Emitted event topic definitions |
| Types | `src/types.rs` | Data structures (Transfer, BatchOperation, etc.) |

## Prior ADRs

Design decisions relevant to audit preparedness are recorded in
[`docs/adr/`](adr/):

| ADR | Title |
|---|---|
| 0001 | [Use Soroban SDK 21.x](adr/0001-use-soroban-sdk-21-x.md) |
| 0007 | [Establish an event naming convention](adr/0007-establish-an-event-naming-convention.md) |
| 0009 | [Use checked arithmetic for all math](adr/0009-use-checked-arithmetic-for-all-math.md) |
| 0010 | [Represent token amounts as i128](adr/0010-represent-token-amounts-as-i128.md) |
| 0011 | [Introduce an admin role](adr/0011-introduce-an-admin-role.md) |
| 0012 | [Provide a pause and unpause mechanism](adr/0012-provide-a-pause-and-unpause-mechanism.md) |
| 0017 | [Use TestUtils for contract tests](adr/0017-use-testutils-for-contract-tests.md) |
| 0020 | [Optimize the release wasm binary](adr/0020-optimize-the-release-wasm-binary.md) |
| 0037 | [Set up a CI approach for tests](adr/0037-set-up-a-ci-approach-for-tests.md) |
| 0038 | [Prepare artifacts for a security audit](adr/0038-prepare-artifacts-for-a-security-audit.md) |

