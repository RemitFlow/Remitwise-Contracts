default: build

build:
	cargo build --target wasm32-unknown-unknown --release

optimize: build
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/remitflow_contract.wasm

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

.PHONY: default build optimize test fmt lint clean
