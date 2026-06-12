default: build

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

.PHONY: default build test fmt lint clean
