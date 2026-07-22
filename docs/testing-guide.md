# Testing Guide

Run the contract test suite from the repository root:

```sh
cargo test
```

See the README and the sources under src/ for the authoritative implementation.
## Running Tests

Execute all tests with:

```bash
make test
```

Run tests with verbose output:

```bash
cargo test -- --nocapture
```

Run a specific test:

```bash
cargo test test_pause_requires_admin_auth -- --nocapture
```

## Admin-Only Guards Testing

The RemitFlow contract implements several admin-only guards to ensure only the administrator can perform sensitive operations. These guards are tested comprehensively to ensure authorization is properly enforced.

### What Are Admin-Only Guards?

Admin-only guards are authorization checks that restrict certain contract operations to the administrator address only. These operations include:

1. **initialize()** - Sets up the contract with an admin and token address
2. **pause()** - Blocks creation of new transfers
3. **unpause()** - Re-enables transfer creation

### Testing Admin Authorization

Tests verify that admin-only operations:
- **Succeed when called by the admin** with proper authorization
- **Fail when called by non-admin addresses** without authorization
- **Fail if the contract is not yet initialized** (NotInitialized error)
- **Cannot be called twice** when appropriate (AlreadyInitialized for initialize())

### Key Test Scenarios

#### Initialization Guard
```rust
#[test]
fn test_initialize_twice_fails() {
    // Ensures AlreadyInitialized prevents re-initialization with same parameters
}

#[test]
fn test_reinitialize_with_different_admin_and_token_fails() {
    // Ensures re-initialization fails when supplying different admin/token parameters
}

#[test]
fn test_reinitialize_after_admin_transfer_fails() {
    // Verifies re-initialization attempts fail after 2-step admin transfer completes
}

#[test]
fn test_reinitialize_after_active_transfers_and_state_changes_fails() {
    // Verifies re-initialization attempts fail after transfer creation and pause state changes
}

#[test]
fn test_reinitialize_by_unauthorized_caller_fails() {
    // Ensures unauthorized callers receive AlreadyInitialized on already initialized contracts
}

#[test]
fn test_reinitialize_does_not_emit_init_event() {
    // Ensures failed re-initialization attempts do not emit duplicate init events
}
```

#### Pause/Unpause Guards
```rust
#[test]
fn test_pause_requires_admin_auth() {
    // Verifies pause() checks admin.require_auth()
}

#[test]
fn test_unpause_requires_admin_auth() {
    // Verifies unpause() checks admin.require_auth()
}
```

#### Operational State Guards
```rust
#[test]
fn test_admin_operations_require_initialization() {
    // Ensures pause, unpause, and other admin ops fail when contract is not initialized
}
```

### Storage TTL Bump Testing

Tests verify that Time-To-Live (TTL) bump logic functions correctly across storage tiers:

```rust
#[test]
fn test_ttl_bump_constants_configured_correctly() {
    // Verifies INSTANCE_BUMP_THRESHOLD/AMOUNT and PERSISTENT_BUMP_THRESHOLD/AMOUNT constants
}

#[test]
fn test_instance_ttl_bumped_on_mutating_calls() {
    // Verifies instance storage TTL extensions on mutating contract entrypoints
}

#[test]
fn test_persistent_ttl_bumped_on_transfer_and_caller_writes() {
    // Verifies persistent storage TTL extensions when writing transfers and allowed callers
}

#[test]
fn test_cancel_transfer_bumps_persistent_ttl() {
    // Ensures cancelling an expired transfer extends persistent storage TTL
}

#[test]
fn test_admin_transfer_flow_bumps_instance_ttl() {
    // Verifies instance TTL bumps during 2-step admin ownership transfer
}
```


#[test]
fn test_pause_and_unpause_state_changes() {
    // Validates pause/unpause state properly gates transfer creation
}
```

### Test Execution Patterns

#### Pattern 1: Testing with Mocked Auth
The test harness uses `env.mock_all_auths()` to automatically approve all authorization checks. This is useful for positive tests:

```rust
let s = setup(); // Creates environment with mocked auth
s.client.pause(); // Admin auth is auto-approved
assert!(s.client.is_paused());
```

#### Pattern 2: Testing Without Mocked Auth
To test authorization failures, create a fresh environment without auth mocking:

```rust
let env = Env::default(); // No mocked auth
let admin = Address::generate(&env);
// ... initialize contract ...
let res = client.try_pause(); // Will fail - no auth provided
assert!(res.is_err());
```

### Authentication Mechanisms in Soroban

RemitFlow uses the Soroban SDK's `require_auth()` method on the Address type. When called:
- The SDK checks if the address has authorized the current contract invocation
- If auth is missing, the contract invocation fails
- `mock_all_auths()` bypasses this check for testing purposes

### Common Test Patterns

**Testing Success Cases:**
1. Call setup() to get a mocked environment
2. Invoke the admin operation (auth is auto-approved)
3. Assert the operation succeeded and state changed appropriately

**Testing Authorization Failures:**
1. Create a fresh Env::default() (no mocked auth)
2. Generate addresses and initialize contract
3. Call admin operation via try_* variant
4. Assert the result is an error

**Testing State Constraints:**
1. Setup contract in a particular state (paused, initialized, etc.)
2. Attempt an operation that should be blocked
3. Verify the appropriate error is returned

## Integration with CI/CD

All tests are automatically run as part of the project's continuous integration pipeline:

```bash
# From Makefile
make test
```

This ensures admin guards remain properly enforced across code changes and refactorings.
## Common test setup

Contract tests should use `TestFixture` from `src/test_utils.rs`. Calling
`TestFixture::new()`:

- creates an isolated Soroban `Env` with mocked authorization;
- generates admin, sender, and recipient addresses;
- deploys a Stellar Asset Contract and funds the sender;
- deploys and initializes the RemitFlow contract; and
- exposes the environment, contract client, token address, and actors.

The fixture also provides focused helpers for setup repeated across lifecycle
tests:

- `token_client()` returns a client for balance assertions;
- `future_expiry()` returns a valid expiry relative to the ledger time; and
- `create_default_transfer()` creates a standard pending transfer.

Prefer these defaults when the values are not relevant to the behavior under
test. Use the fixture's public test fields and contract client directly when a
test needs non-default actors, amounts, expiry, or ledger state. Keep helpers
limited to setup and avoid hiding the action or assertion that defines a test.
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
