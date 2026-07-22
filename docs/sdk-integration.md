# SDK Compatibility

RemitFlow currently targets `soroban-sdk` **21.7.6**. Both the runtime dependency
and the `testutils` development dependency are pinned to the exact requirement
`=21.7.6` in `Cargo.toml`, and `Cargo.lock` resolves `soroban-sdk` to 21.7.6.

The exact pin is intentional. It keeps local builds and CI on the SDK version the
contract has been tested against and prevents Cargo from selecting a newer 21.x
release without an explicit compatibility review. The change that introduced the
pin did not upgrade the SDK: it changed the requirement from `21.7.6` (Cargo's
caret-compatible syntax) to `=21.7.6` while retaining the same resolved version.

## Compatibility considerations

The repository does not currently record a migration from an older SDK release,
so there are no version-to-version breaking changes to report for the present
21.7.6 target. Before moving away from 21.7.6, contributors must review the
upstream release notes for every version being crossed rather than assuming that
another 21.x release is compatible.

Pay particular attention to changes that affect:

- contract and client macro expansion or generated interfaces;
- serialization of contract types, storage keys, events, errors, and contract
  metadata;
- authorization and token-client behavior;
- ledger protocol, host-function, or Stellar CLI compatibility;
- resource metering, TTL, and rent behavior; and
- `testutils` APIs or test behavior.

Changes in these areas can affect the contract ABI, persisted on-chain state,
deployment tooling, resource usage, or tests even when the Rust code still
compiles. Storage key variant names are part of the on-chain storage interface;
see [Storage Model](storage-model.md#upgrade-notes) before accepting any
serialization-related change.

## Safe upgrade procedure

1. Read the upstream Soroban SDK release notes for the complete range from
   21.7.6 to the proposed version. Record relevant breaking changes and the
   matching Stellar network protocol and CLI requirements in the pull request.
2. Update both `soroban-sdk` entries in `Cargo.toml` to the same exact version.
   Keep the leading `=` so upgrades remain explicit.
3. Regenerate `Cargo.lock` with the intended dependency update and inspect the
   Soroban and Stellar transitive dependency changes. Do not edit the lockfile by
   hand.
4. Apply required source, test, deployment, ABI, storage, event, and metadata
   migrations. If persisted representations change, document and test the
   on-chain migration before deployment.
5. Run the repository's compatibility checks:

   ```sh
   cargo fmt --all -- --check
   cargo build --release --target wasm32-unknown-unknown
   cargo test --locked
   cargo clippy --all-targets --locked -- -D warnings
   ```

6. Compare the release WASM and its resource usage with the current baseline.
   If contract interfaces or serialized values changed, exercise existing
   deployments or representative ledger snapshots as part of an integration
   test.
7. Upgrade the Stellar CLI and deployment environment only when required by the
   selected SDK/protocol combination, then verify deployment and invocation on a
   compatible local network or testnet before mainnet rollout.

An SDK upgrade should be submitted as a focused pull request containing the
manifest and lockfile changes, compatibility findings, any required migrations,
and the resulting test and WASM checks.
