default: build

build:
	cargo build --target wasm32-unknown-unknown --release

check-wasm-size: build
	./scripts/check-wasm-size.sh

test-wasm-size-check:
	./scripts/test-check-wasm-size.sh

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

# Deploy and initialize the contract on a Stellar network.
# Required variables: SOURCE, ADMIN, TOKEN
# Optional: NETWORK (default: testnet), WASM, SKIP_BUILD
# Example:
#   make deploy NETWORK=testnet SOURCE=my-key ADMIN=G... TOKEN=C...
deploy:
	./scripts/deploy-and-initialize.sh \
		--network $(or $(NETWORK),testnet) \
		--source $(SOURCE) \
		--admin $(ADMIN) \
		--token $(TOKEN) \
		$(if $(WASM),--wasm $(WASM),) \
		$(if $(SKIP_BUILD),--skip-build,)

.PHONY: default build check-wasm-size test-wasm-size-check optimize test coverage coverage-lcov fmt fmt-check lint doc clean deploy
