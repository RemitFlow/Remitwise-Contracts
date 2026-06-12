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
