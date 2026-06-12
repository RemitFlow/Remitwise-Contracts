default: build

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

clean:
	cargo clean

.PHONY: default build test clean
