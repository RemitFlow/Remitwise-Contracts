#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Address;

use crate::test_utils::{
    TestFixture, DEFAULT_SENDER_BALANCE, DEFAULT_TRANSFER_AMOUNT,
};
use crate::types::Status;

fn setup<'a>() -> TestFixture<'a> {
    TestFixture::new()
}

#[test]
fn test_initialize_sets_admin_and_token() {
    let s = setup();
    assert_eq!(s.client.get_admin(), s.admin);
    assert_eq!(s.client.get_token(), s.token);
    assert_eq!(s.client.counter(), 0);
}

#[test]
fn test_common_setup_initializes_and_funds_contract() {
    let s = setup();

    assert_eq!(s.client.get_admin(), s.admin);
    assert_eq!(s.client.get_token(), s.token);
    assert_eq!(s.token_client().balance(&s.from), DEFAULT_SENDER_BALANCE);
    assert_eq!(s.token_client().balance(&s.client.address), 0);
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
    let token_client = s.token_client();

    let id = s.create_default_transfer();

    assert_eq!(id, 1);
    assert_eq!(s.client.counter(), 1);
    assert_eq!(
        token_client.balance(&s.from),
        DEFAULT_SENDER_BALANCE - DEFAULT_TRANSFER_AMOUNT
    );
    assert_eq!(
        token_client.balance(&s.client.address),
        DEFAULT_TRANSFER_AMOUNT
    );

    let transfer = s.client.get_transfer(&id);
    assert_eq!(transfer.amount, DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(transfer.status, Status::Pending);
    assert_eq!(transfer.recipient, s.recipient);
}

#[test]
fn test_create_transfer_rejects_non_positive_amount() {
    let s = setup();
    let expiry = s.future_expiry();
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
    let token_client = s.token_client();
    let id = s.create_default_transfer();

    s.client.claim_transfer(&id, &s.recipient);

    assert_eq!(
        token_client.balance(&s.recipient),
        DEFAULT_TRANSFER_AMOUNT
    );
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
    let token_client = s.token_client();
    let expiry = s.future_expiry();
    let id = s.create_default_transfer();

    s.env.ledger().with_mut(|l| l.timestamp = expiry + 1);
    s.client.cancel_transfer(&id, &s.from);

    assert_eq!(
        token_client.balance(&s.from),
        DEFAULT_SENDER_BALANCE
    );
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
