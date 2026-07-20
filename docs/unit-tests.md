# Unit Tests

This note documents the **unit-tests** of the remitflow-contract contract.

remitflow-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the unit-tests in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.
## Test Coverage Overview

The contract includes comprehensive unit tests organized by functionality:

### Initialization Tests
- **test_initialize_sets_admin_and_token**: Verifies admin and token are correctly stored during initialization
- **test_initialize_twice_fails**: Ensures contract cannot be re-initialized after first setup

### Transfer Creation Tests
- **test_create_transfer_moves_funds_to_escrow**: Validates token movement into escrow during transfer creation
- **test_create_transfer_rejects_non_positive_amount**: Enforces positive amount requirement
- **test_create_transfer_rejects_past_expiry**: Prevents creating transfers with past expiry times
- **test_create_transfer_rejects_self_transfer**: Prevents sender from being the recipient
- **test_create_transfer_rejects_oversized_amount**: Enforces maximum amount limits
- **test_create_transfer_rejects_far_future_expiry**: Prevents excessive future expiry dates

### Transfer Claim Tests
- **test_claim_transfer_pays_recipient**: Verifies recipient receives funds upon claim
- **test_claim_transfer_wrong_recipient_fails**: Ensures only authorized recipient can claim
- **test_claim_after_expiry_fails**: Prevents claiming expired transfers
- **test_claim_twice_fails**: Ensures transfer cannot be claimed twice

### Transfer Cancellation Tests
- **test_cancel_after_expiry_refunds_sender**: Verifies sender receives refund after expiry
- **test_cancel_before_expiry_fails**: Prevents early cancellation
- **test_cancel_by_non_sender_fails**: Ensures only original sender can cancel

### Query and Utility Tests
- **test_get_unknown_transfer_fails**: Validates error handling for non-existent transfers
- **test_counter_increments_across_transfers**: Verifies transfer ID counter increments correctly
- **test_total_escrowed_tracks_pending_amounts**: Validates accurate escrow balance calculation
- **test_count_for_sender_and_recipient**: Tests sender and recipient transfer counting
- **test_get_transfers_paged_respects_limit_and_start**: Validates pagination functionality
- **test_is_expired_reflects_ledger_time**: Verifies expiry status matches ledger time
- **test_count_by_status_tracks_lifecycle**: Tests transfer status counting

### Reusable Math Module Tests
- Checked amount addition and subtraction at normal and overflow boundaries
- Checked counter increments through `u64::MAX`
- Saturating amount and capped-counter behavior
- Basis-point fees at zero, common rates, 100%, rounding boundaries, and
  `i128::MAX`
- Rejection of negative amounts and rates above 10,000 basis points

### Pause/Unpause Tests
- **test_pause_blocks_create_transfer**: Verifies pause prevents new transfer creation
- **test_pause_by_admin_succeeds**: Validates admin can pause the contract
- **test_unpause_by_admin_succeeds**: Validates admin can unpause the contract
- **test_pause_and_unpause_state_changes**: Tests complete pause/unpause lifecycle

## Admin-Only Guard Tests

These tests specifically validate authorization mechanisms for admin-only operations:

### Initialization Authorization
- **test_initialize_requires_admin_auth**: Verifies initialize() enforces admin authentication
- **test_initialize_by_admin_succeeds**: Confirms admin can successfully initialize
- **test_initialize_only_once_enforces_admin_guard**: Validates admin guard prevents re-initialization

### Pause/Unpause Authorization
- **test_pause_requires_admin_auth**: Verifies pause() enforces admin authentication
- **test_unpause_requires_admin_auth**: Verifies unpause() enforces admin authentication
- **test_admin_guard_on_pause_with_mock_all_auths**: Tests pause/unpause with authentication mocking

### Operational Constraints
- **test_admin_operations_require_initialization**: Ensures admin operations fail if contract not initialized
- **test_non_admin_cannot_pause_twice**: Tests state consistency for pause operations

## Test Infrastructure

The test suite uses:
- **Setup struct**: Bundles contract client, token, and test addresses
- **create_token()**: Helper to deploy Stellar Asset Contract for testing
- **setup()**: Initializes fully configured contract with funded sender
- **mock_all_auths()**: Simplifies testing by auto-approving all authorization checks
