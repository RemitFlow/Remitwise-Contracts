# Reproducible Builds

This guide explains how to produce **bit-identical WASM binaries** from the
RemitFlow source code across different machines, CI runners, and points in
time.

A build is *reproducible* when compiling the same commit twice produces two
byte-identical artifacts. This property lets anyone verify that the deployed
contract matches the audited source, without trusting a single build server.

## Why this matters

Stellar/Soroban contracts are deployed as WASM blobs. When you invoke

```sh
stellar contract deploy --wasm remitflow_contract.wasm
```

the network stores that blob. Reproducible builds let a third party:

- confirm that the **deployed WASM matches a given commit**;
- **detect supply-chain tampering** (a compromised CI or developer machine
  would produce a different hash);
- allow **auditors and reviewers** to build and compare without access to the
  original deployment environment.

## Determinism guarantees

The following project settings combine to make the build reproducible:

| Setting | Location | Effect |
|---|---|---|
| Pinned Rust toolchain | [`rust-toolchain.toml`](../rust-toolchain.toml) | Every build uses **exactly** `rustc 1.90.0`, regardless of what the host system has installed. |
| Locked dependencies | [`Cargo.lock`](../Cargo.lock) | Version resolution is frozen — `cargo build --locked` fails if the lock file is stale, preventing accidental version drift. |
| Single codegen unit | `Cargo.toml` → `codegen-units = 1` | The compiler uses one codegen unit, which removes parallelism-induced non-determinism in symbol ordering. |
| LTO enabled | `Cargo.toml` → `lto = true` | Link-time optimisation runs deterministically when combined with a single codegen unit. |
| Optimisation level | `Cargo.toml` → `opt-level = "z"` | Prioritises binary size; the same input always produces the same output. |
| Overflow checks | `Cargo.toml` → `overflow-checks = true` | On-by-default for debug; explicitly set in release so the flag is documented. |
| Debug stripped | `Cargo.toml` → `strip = "symbols"` | Ensures symbol ordering does not vary across builds. |
| Panic abort | `Cargo.toml` → `panic = "abort"` | Removes unwind tables that can introduce section ordering variation. |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` | Cross-compilation target — the host architecture does not affect the output. |

## Producing a reproducible WASM

### Prerequisites

- Rust toolchain manager ([rustup](https://rustup.rs/))
- The `wasm32-unknown-unknown` target

### Steps

```sh
# 1. Ensure the pinned toolchain is installed
rustup toolchain install 1.90.0

# 2. Add the WASM target
rustup target add wasm32-unknown-unknown

# 3. Clone at the exact commit
git clone https://github.com/RemitFlow/Remitwise-Contracts.git
cd Remitwise-Contracts
git checkout <commit-hash-or-tag>

# 4. Build with locked dependencies
cargo build --locked --target wasm32-unknown-unknown --release
```

> [!IMPORTANT]
> The `--locked` flag is **required**. Without it, Cargo may resolve
> dependency versions differently, producing a different WASM. CI enforces
> `--locked` in the build step — see [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

The compiled artifact is written to:

```
target/wasm32-unknown-unknown/release/remitflow_contract.wasm
```

### Build via Makefile

The `Makefile` build target is equivalent:

```sh
make build
```

This also passes `--locked` implicitly when the project is set up correctly,
but for verification purposes prefer the explicit `cargo build --locked`
invocation above.

## Verifying a deployed contract

After deployment, use the bundled verification script to check that the
on-chain WASM matches your local build:

```sh
./scripts/verify-wasm-hash.sh testnet <CONTRACT_ID>
```

The script:

1. Builds the WASM if it is not already present (with `--locked`);
2. Computes a SHA-256 hash of the local WASM file;
3. Fetches the on-chain WASM hash via `stellar contract inspect`;
4. Compares the two — exit code `0` on match, non-zero on mismatch.

### Manual hash check

To verify without a network connection or the Stellar CLI:

```sh
sha256sum target/wasm32-unknown-unknown/release/remitflow_contract.wasm
```

Compare this hash against a trusted party's output for the same commit.

## CI and reproducibility

The CI pipeline (`.github/workflows/ci.yml`) builds with `--locked` and the
pinned toolchain. This means a CI build that passes is already a candidate for
a reproducible artifact. The CI does **not** currently publish the WASM hash
as a build artifact — if you need this, consider adding a step:

```yaml
- name: Compute WASM hash
  run: |
    sha256sum target/wasm32-unknown-unknown/release/remitflow_contract.wasm \
      > wasm-hash.txt
- uses: actions/upload-artifact@v4
  with:
    name: wasm-hash
    path: wasm-hash.txt
```

## Troubleshooting

### "I get a different hash on macOS / Windows"

Cross-platform variance can affect reproducibility. The pinned Rust toolchain
eliminates compiler differences, but linker behaviour may still differ. For
the most reliable results:

- **Use Linux** (Ubuntu, Debian, or the same base image as CI).
- Use the same `cargo` and `rustc` version (the toolchain file enforces this).
- Run inside the CI Docker image if available.
- Future work may add a Docker-based build container for fully hermetic builds.

### "The `stellar contract inspect` command failed"

Ensure you are on Stellar CLI version that supports the network profile and
that the contract ID is correct. See the [deployment guide](deployment-guide.md)
for CLI setup instructions.

### "I modified `Cargo.toml` or `Cargo.lock`"

Any change to dependencies, their versions, or the release profile will change
the WASM hash. This is expected — re-verify after every dependency update.
