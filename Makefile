default: build

build:
	cargo build --target wasm32-unknown-unknown --release

clean:
	cargo clean

.PHONY: default build clean
