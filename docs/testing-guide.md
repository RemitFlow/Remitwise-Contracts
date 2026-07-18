# Testing Guide

RemitFlow's tests are Rust unit tests backed by the Soroban SDK test utilities.
Run the complete suite from the repository root:

```sh
make test
```

## Coverage

Coverage is collected with
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov). It uses LLVM's
source-based instrumentation, works with the repository's pinned Rust
toolchain, and excludes dependencies from the report by default.

Install the command once:

```sh
cargo install cargo-llvm-cov --locked
```

The required `llvm-tools-preview` component is declared in
`rust-toolchain.toml`, so rustup installs it with the pinned toolchain.

Generate a browsable HTML report:

```sh
make coverage
```

The entry page is `target/llvm-cov/html/index.html`.

Generate an LCOV file for editors or other reporting services:

```sh
make coverage-lcov
```

The result is `target/llvm-cov/lcov.info`. Both commands run the full test
suite while collecting coverage, so a failing test also makes the command
fail.

The CI coverage job runs on pushes to `main` and on pull requests. It publishes
the HTML and LCOV reports as the `coverage-report` workflow artifact and writes
a coverage summary to the job log. Coverage output lives under the ignored
`target/` directory and should not be committed.
