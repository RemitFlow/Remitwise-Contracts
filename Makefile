default: build

build:
	cargo build --target wasm32-unknown-unknown --release

optimize: build
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/remitflow_contract.wasm

test:
	cargo test

coverage:
	cargo llvm-cov --workspace --all-features --html --output-dir target/llvm-cov/html

coverage-lcov:
	cargo llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/lcov.info

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets -- -D warnings

doc:
	cargo doc --no-deps

clean:
	cargo clean

.PHONY: default build optimize test coverage coverage-lcov fmt fmt-check lint doc clean
