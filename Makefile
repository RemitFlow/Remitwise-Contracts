default: build

build:
	cargo build --target wasm32-unknown-unknown --release

optimize: build
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/remitflow_contract.wasm

test:
	cargo test

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

.PHONY: default build optimize test fmt fmt-check lint doc clean
