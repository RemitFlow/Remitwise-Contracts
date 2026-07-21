# WASM Size Budget

The compiled RemitFlow contract must be no larger than **65,536 bytes (64
KiB)**. This is the project's maximum size for the release WASM artifact and
is enforced as a hard limit: an artifact of exactly 65,536 bytes passes, while
an artifact of 65,537 bytes fails.

The budget applies to the release artifact at:

```text
target/wasm32-unknown-unknown/release/remitflow_contract.wasm
```

It does not apply to the Rust source tree, intermediate build files, debug
artifacts, or the size of a compressed archive. Stellar network limits are
protocol configuration and may change independently; contributors should
confirm the destination network's current configuration before deployment.

## How the budget is measured

`scripts/check-wasm-size.sh` measures the artifact's exact byte count with
`wc -c` and compares it with `WASM_SIZE_BUDGET_BYTES`, which defaults to
`65536`. CI runs the checker immediately after the pinned release build, so a
pull request cannot add an oversized contract without failing the build.

Run the same check locally with:

```sh
make check-wasm-size
```

That command first produces the canonical release build and then reports the
current size, the budget, and the remaining headroom. To inspect another
already-built artifact directly:

```sh
./scripts/check-wasm-size.sh path/to/contract.wasm
```

The environment override exists for testing the checker and evaluating a
proposed future budget. It must not be used to bypass the default limit in CI:

```sh
WASM_SIZE_BUDGET_BYTES=70000 ./scripts/check-wasm-size.sh path/to/contract.wasm
```

Changes to the official 65,536-byte budget must update the script default and
this document together, with the rationale recorded in the pull request.

## Guidance for contributors

Check size before and after adding functionality and include the byte delta in
the pull request description. Keep the release profile's size-oriented
settings (`opt-level = "z"`, LTO, one codegen unit, stripped symbols, and abort
on panic) intact unless a measured alternative is smaller and passes all
tests.

When a change adds significant size:

- Prefer existing Soroban SDK and internal helpers over duplicate logic.
- Avoid large static tables, embedded strings, and dependencies used for only
  a small amount of functionality.
- Keep diagnostics and test-only helpers behind test configuration so they do
  not enter the release artifact.
- Use `cargo tree` to review newly introduced transitive dependencies.
- Split unrelated functionality into another contract if the feature cannot
  fit with reasonable refactoring.

Do not optimize solely by deleting validation, authorization, overflow checks,
or useful contract errors. Security and correctness remain requirements; if a
necessary feature does not fit, raise the tradeoff with maintainers rather
than weakening those controls.
