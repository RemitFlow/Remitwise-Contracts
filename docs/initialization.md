# Initialization

This document details the **initialization** process and re-initialization guards of the RemitFlow smart contract.

## Overview

The `initialize` entrypoint sets up the core contract parameters:
- **Admin (`Address`)**: Designated administrator key with privilege to manage contract pause state and allowlists.
- **Token (`Address`)**: Designated Stellar Asset Contract token address used for all escrowed transfers.

## Single-Initialization Invariant

To prevent contract takeover or state corruption:
1. `initialize` checks `storage::has_admin(&env)` prior to processing inputs or requiring auth.
2. If `storage::has_admin(&env)` evaluates to `true`, the contract immediately aborts with `Error::AlreadyInitialized`.
3. No storage modifications, admin updates, token updates, or event emissions occur during a failed re-initialization attempt.

## Protection against Re-Initialization Scenarios

The contract guards against re-initialization across all execution contexts:
- **Same Parameters**: Calling `initialize` again with existing admin and token fails with `AlreadyInitialized`.
- **Different Parameters**: Calling `initialize` with new admin or token addresses fails with `AlreadyInitialized`.
- **Post Admin-Rotation**: Attempting `initialize` after a 2-step admin transfer (`transfer_admin` -> `accept_admin`) fails with `AlreadyInitialized`.
- **Post State Transitions**: Attempting `initialize` after transfer creation, pause state updates, or allowlist mutations fails with `AlreadyInitialized`.
- **Unauthorized Callers**: Calling `initialize` as a non-admin third party fails with `AlreadyInitialized`.

## Automated Test Coverage

Re-initialization protection is tested in `src/test.rs`:
- `test_initialize_twice_fails`
- `test_reinitialize_with_different_admin_and_token_fails`
- `test_reinitialize_after_admin_transfer_fails`
- `test_reinitialize_after_active_transfers_and_state_changes_fails`
- `test_reinitialize_by_unauthorized_caller_fails`
- `test_reinitialize_does_not_emit_init_event`

