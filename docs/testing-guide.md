# Testing Guide

Run the contract test suite from the repository root:

```sh
cargo test
```

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
