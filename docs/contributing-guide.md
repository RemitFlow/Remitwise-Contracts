# Contributing Guide

Before opening a pull request, run the same checks enforced by CI:

```sh
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

Clippy warnings are treated as errors. New or changed Rust code must therefore
compile without warnings across every target, including tests.
