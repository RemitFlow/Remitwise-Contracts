# Testing Guide

This note documents the **testing-guide** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the testing-guide in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

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
fn test_initialize_requires_admin_auth() {
    // Verifies initialize() checks admin.require_auth()
}

#[test]
fn test_initialize_only_once_enforces_admin_guard() {
    // Ensures AlreadyInitialized prevents re-initialization
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
    // Ensures admin operations fail if contract not initialized
}

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