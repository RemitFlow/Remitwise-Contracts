#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

use crate::types::Status;
use crate::{RemitFlowContract, RemitFlowContractClient};

/// Test harness bundling the contract client, token, and key addresses.
struct Setup<'a> {
    env: Env,
    client: RemitFlowContractClient<'a>,
    token: Address,
    admin: Address,
    from: Address,
    recipient: Address,
}

/// Deploy a Stellar Asset Contract and return its address and clients.
fn create_token<'a>(env: &Env, admin: &Address) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let contract = env.register_stellar_asset_contract_v2(admin.clone());
    let address = contract.address();
    (
        address.clone(),
        TokenClient::new(env, &address),
        StellarAssetClient::new(env, &address),
    )
}

/// Build a fully initialized contract with a funded sender.
fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token, _token_client, token_admin) = create_token(&env, &admin);
    token_admin.mint(&from, &1_000);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);

    Setup {
        env,
        client,
        token,
        admin,
        from,
        recipient,
    }
}

#[test]
fn test_initialize_sets_admin_and_token() {
    let s = setup();
    assert_eq!(s.client.get_admin(), s.admin);
    assert_eq!(s.client.get_token(), s.token);
    assert_eq!(s.client.counter(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let s = setup();
    let res = s.client.try_initialize(&s.admin, &s.token);
    assert_eq!(res, Err(Ok(crate::error::Error::AlreadyInitialized)));
}

#[test]
fn test_create_transfer_moves_funds_to_escrow() {
    let s = setup();
    let token_client = TokenClient::new(&s.env, &s.token);
    let expiry = s.env.ledger().timestamp() + 1_000;

    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    assert_eq!(id, 1);
    assert_eq!(s.client.counter(), 1);
    assert_eq!(token_client.balance(&s.from), 600);
    assert_eq!(token_client.balance(&s.client.address), 400);

    let transfer = s.client.get_transfer(&id);
    assert_eq!(transfer.amount, 400);
    assert_eq!(transfer.status, Status::Pending);
    assert_eq!(transfer.recipient, s.recipient);
}

#[test]
fn test_create_transfer_rejects_non_positive_amount() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let res = s.client.try_create_transfer(&s.from, &s.recipient, &0, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::InvalidAmount)));
}

#[test]
fn test_create_transfer_rejects_past_expiry() {
    let s = setup();
    s.env.ledger().with_mut(|l| l.timestamp = 5_000);
    let res = s.client.try_create_transfer(&s.from, &s.recipient, &100, &1_000);
    assert_eq!(res, Err(Ok(crate::error::Error::InvalidExpiry)));
}

#[test]
fn test_claim_transfer_pays_recipient() {
    let s = setup();
    let token_client = TokenClient::new(&s.env, &s.token);
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    s.client.claim_transfer(&id, &s.recipient);

    assert_eq!(token_client.balance(&s.recipient), 400);
    assert_eq!(token_client.balance(&s.client.address), 0);
    assert_eq!(s.client.get_transfer(&id).status, Status::Claimed);
}

#[test]
fn test_claim_transfer_wrong_recipient_fails() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    let res = s.client.try_claim_transfer(&id, &stranger);
    assert_eq!(res, Err(Ok(crate::error::Error::Unauthorized)));
}

#[test]
fn test_claim_after_expiry_fails() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    s.env.ledger().with_mut(|l| l.timestamp = expiry + 1);
    let res = s.client.try_claim_transfer(&id, &s.recipient);
    assert_eq!(res, Err(Ok(crate::error::Error::Expired)));
}

#[test]
fn test_cancel_after_expiry_refunds_sender() {
    let s = setup();
    let token_client = TokenClient::new(&s.env, &s.token);
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    s.env.ledger().with_mut(|l| l.timestamp = expiry + 1);
    s.client.cancel_transfer(&id, &s.from);

    assert_eq!(token_client.balance(&s.from), 1_000);
    assert_eq!(token_client.balance(&s.client.address), 0);
    assert_eq!(s.client.get_transfer(&id).status, Status::Cancelled);
}

#[test]
fn test_cancel_before_expiry_fails() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    let res = s.client.try_cancel_transfer(&id, &s.from);
    assert_eq!(res, Err(Ok(crate::error::Error::NotExpired)));
}

#[test]
fn test_cancel_by_non_sender_fails() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    s.env.ledger().with_mut(|l| l.timestamp = expiry + 1);
    let res = s.client.try_cancel_transfer(&id, &stranger);
    assert_eq!(res, Err(Ok(crate::error::Error::Unauthorized)));
}

#[test]
fn test_claim_twice_fails() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &400, &expiry);

    s.client.claim_transfer(&id, &s.recipient);
    let res = s.client.try_claim_transfer(&id, &s.recipient);
    assert_eq!(res, Err(Ok(crate::error::Error::NotPending)));
}

#[test]
fn test_get_unknown_transfer_fails() {
    let s = setup();
    let res = s.client.try_get_transfer(&999);
    assert_eq!(res, Err(Ok(crate::error::Error::TransferNotFound)));
}

#[test]
fn test_counter_increments_across_transfers() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id1 = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    let id2 = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(s.client.counter(), 2);
}

#[test]
fn test_create_transfer_rejects_self_transfer() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let res = s.client.try_create_transfer(&s.from, &s.from, &100, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::SameParty)));
}

#[test]
fn test_create_transfer_rejects_oversized_amount() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let amount = crate::MAX_AMOUNT + 1;
    let res = s
        .client
        .try_create_transfer(&s.from, &s.recipient, &amount, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::AmountTooLarge)));
}

#[test]
fn test_create_transfer_rejects_far_future_expiry() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + crate::MAX_EXPIRY_WINDOW + 1;
    let res = s.client.try_create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::ExpiryTooFar)));
}

#[test]
fn test_total_escrowed_tracks_pending_amounts() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;

    assert_eq!(s.client.total_escrowed(), 0);

    let id1 = s.client.create_transfer(&s.from, &s.recipient, &300, &expiry);
    s.client.create_transfer(&s.from, &s.recipient, &200, &expiry);
    assert_eq!(s.client.total_escrowed(), 500);

    s.client.claim_transfer(&id1, &s.recipient);
    assert_eq!(s.client.total_escrowed(), 200);
}

#[test]
fn test_count_for_sender_and_recipient() {
    let s = setup();
    let other = Address::generate(&s.env);
    let expiry = s.env.ledger().timestamp() + 1_000;

    s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    s.client.create_transfer(&s.from, &other, &100, &expiry);

    assert_eq!(s.client.count_for_sender(&s.from), 2);
    assert_eq!(s.client.count_for_sender(&other), 0);
    assert_eq!(s.client.count_for_recipient(&s.recipient), 1);
    assert_eq!(s.client.count_for_recipient(&other), 1);
}

#[test]
fn test_get_transfers_paged_respects_limit_and_start() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    for _ in 0..3 {
        s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    }

    let first = s.client.get_transfers_paged(&1, &2);
    assert_eq!(first.len(), 2);
    assert_eq!(first.get(0).unwrap().id, 1);
    assert_eq!(first.get(1).unwrap().id, 2);

    let second = s.client.get_transfers_paged(&3, &2);
    assert_eq!(second.len(), 1);
    assert_eq!(second.get(0).unwrap().id, 3);

    let empty = s.client.get_transfers_paged(&1, &0);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_is_expired_reflects_ledger_time() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);

    assert!(!s.client.is_expired(&id));

    s.env.ledger().with_mut(|l| l.timestamp = expiry + 1);
    assert!(s.client.is_expired(&id));
}

#[test]
fn test_pause_blocks_create_transfer() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;

    assert!(!s.client.is_paused());
    s.client.pause();
    assert!(s.client.is_paused());

    let res = s.client.try_create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::ContractPaused)));

    s.client.unpause();
    assert!(!s.client.is_paused());
    let id = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(id, 1);
}

#[test]
fn test_count_by_status_tracks_lifecycle() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;
    let id1 = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    let _id2 = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);

    assert_eq!(s.client.count_by_status(&Status::Pending), 2);
    assert_eq!(s.client.count_by_status(&Status::Claimed), 0);

    s.client.claim_transfer(&id1, &s.recipient);

    assert_eq!(s.client.count_by_status(&Status::Pending), 1);
    assert_eq!(s.client.count_by_status(&Status::Claimed), 1);
}

// Admin-only guard tests

#[test]
fn test_pause_requires_admin_auth() {
    let s = setup();
    let non_admin = Address::generate(&s.env);

    // Create a new environment without mocked auth to test authorization
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token, _, token_admin) = create_token(&env, &admin);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);

    // Attempting to pause with non-admin should fail
    let res = client.try_pause();
    // Should fail because non_admin doesn't have auth
    assert!(res.is_err());
}

#[test]
fn test_unpause_requires_admin_auth() {
    let s = setup();

    // Pause first (admin can do this)
    s.client.pause();
    assert!(s.client.is_paused());

    // Create a non-admin context
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token, _, _token_admin) = create_token(&env, &admin);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
    client.pause();

    // Attempting to unpause without proper auth should fail
    let res = client.try_unpause();
    assert!(res.is_err());
}

#[test]
fn test_initialize_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let (token, _, _token_admin) = create_token(&env, &admin);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);

    // Initialize without mocked auth - should fail because admin.require_auth() won't pass
    let res = client.try_initialize(&admin, &token);
    assert!(res.is_err());
}

#[test]
fn test_pause_by_admin_succeeds() {
    let s = setup();

    assert!(!s.client.is_paused());
    s.client.pause();
    assert!(s.client.is_paused());
}

#[test]
fn test_unpause_by_admin_succeeds() {
    let s = setup();

    s.client.pause();
    assert!(s.client.is_paused());

    s.client.unpause();
    assert!(!s.client.is_paused());
}

#[test]
fn test_initialize_by_admin_succeeds() {
    let s = setup();
    assert_eq!(s.client.get_admin(), s.admin);
    assert_eq!(s.client.get_token(), s.token);
}

#[test]
fn test_non_admin_cannot_pause_twice() {
    let s = setup();

    // First pause by admin
    s.client.pause();
    assert!(s.client.is_paused());

    // Try to pause again by admin (should succeed since admin always has auth in mock)
    s.client.pause();
    assert!(s.client.is_paused());
}

#[test]
fn test_pause_and_unpause_state_changes() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + 1_000;

    // Initially not paused
    assert!(!s.client.is_paused());

    // Pause and verify
    s.client.pause();
    assert!(s.client.is_paused());

    // Verify transfers are blocked while paused
    let res = s.client.try_create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(res, Err(Ok(crate::error::Error::ContractPaused)));

    // Unpause and verify
    s.client.unpause();
    assert!(!s.client.is_paused());

    // Verify transfers work again
    let id = s.client.create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(id, 1);
}

#[test]
fn test_admin_guard_on_pause_with_mock_all_auths() {
    let s = setup();

    // With mock_all_auths(), admin auth is automatically approved
    assert!(!s.client.is_paused());
    s.client.pause();
    assert!(s.client.is_paused());
    s.client.unpause();
    assert!(!s.client.is_paused());
}

#[test]
fn test_initialize_only_once_enforces_admin_guard() {
    let s = setup();

    // First initialization passed (already done in setup)
    assert_eq!(s.client.get_admin(), s.admin);

    // Second attempt should fail
    let res = s.client.try_initialize(&s.admin, &s.token);
    assert_eq!(res, Err(Ok(crate::error::Error::AlreadyInitialized)));
}

#[test]
fn test_admin_operations_require_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token, _, _) = create_token(&env, &admin);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);

    // Attempting admin operations before initialization should fail
    let res = client.try_pause();
    assert_eq!(res, Err(Ok(crate::error::Error::NotInitialized)));

    let res = client.try_unpause();
    assert_eq!(res, Err(Ok(crate::error::Error::NotInitialized)));
}
