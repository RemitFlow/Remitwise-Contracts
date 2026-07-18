# Testing Guide

Run the automated test suite with the locked dependency versions:

```sh
cargo test --locked
```

CI also runs Clippy against all targets:

```sh
cargo clippy --all-targets --locked -- -D warnings
```

The `-D warnings` flag promotes every Clippy warning to an error, so a pull
request cannot pass CI while lint warnings remain.
