#![cfg(test)]

//! Integration coverage for the escrow conservation ledger.
//!
//! These tests intentionally drive the public lifecycle entrypoints instead of
//! testing only storage helpers. A conservation bug is useful only if it is
//! caught at the boundary where tokens and transfer status change together.

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{vec, Address};

use crate::test_utils::{TestFixture, DEFAULT_EXPIRY_OFFSET, DEFAULT_TRANSFER_AMOUNT};
use crate::types::{BatchOperation, ClaimTransferOperation, CreateTransferOperation, Status};
use crate::{Error, MAX_TOTAL_ESCROWED};

fn setup() -> TestFixture<'static> {
    TestFixture::new()
}

fn assert_conserved(s: &TestFixture<'_>) {
    let funded = s.client.total_funded();
    let pending = s.client.total_escrowed();
    let released = s.client.total_released();

    assert_eq!(funded, pending + released);
    s.client.check_supply_invariant();
}

fn assert_balances_unchanged(s: &TestFixture<'_>, sender: i128, recipient: i128, escrow: i128) {
    assert_eq!(s.token_client().balance(&s.from), sender);
    assert_eq!(s.token_client().balance(&s.recipient), recipient);
    assert_eq!(s.token_client().balance(&s.client.address), escrow);
}

#[test]
fn conservation_survives_mixed_claim_refund_and_sweep_lifecycle() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let first = s
        .client
        .create_transfer(&s.from, &s.recipient, &125, &expiry);
    let second = s
        .client
        .create_transfer(&s.from, &s.recipient, &275, &expiry);
    let third = s
        .client
        .create_transfer(&s.from, &s.recipient, &50, &expiry);

    assert_eq!(s.client.total_funded(), 450);
    assert_eq!(s.client.total_escrowed(), 450);
    assert_eq!(s.client.total_released(), 0);
    assert_conserved(&s);

    s.client.claim_transfer(&first, &s.recipient);
    assert_eq!(s.client.get_status(&first), Status::Claimed);
    assert_eq!(s.client.total_escrowed(), 325);
    assert_eq!(s.client.total_released(), 125);
    assert_conserved(&s);

    s.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp = expiry + 1);
    s.client.cancel_transfer(&second, &s.from);
    assert_eq!(s.client.get_status(&second), Status::Cancelled);
    assert_eq!(s.client.total_escrowed(), 50);
    assert_eq!(s.client.total_released(), 400);
    assert_conserved(&s);

    s.client.sweep_expired(&third);
    assert_eq!(s.client.get_status(&third), Status::Cancelled);
    assert_eq!(s.client.total_funded(), 450);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_released(), 450);
    assert_eq!(s.token_client().balance(&s.client.address), 0);
    assert_conserved(&s);
}

#[test]
fn conservation_uses_each_transfer_amount_without_cross_record_leakage() {
    let s = setup();
    StellarAssetClient::new(&s.env, &s.token).mint(&s.from, &1_000);
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let amounts = [17_i128, 203, 701];
    let mut ids = vec![&s.env];

    for amount in amounts {
        ids.push_back(
            s.client
                .create_transfer(&s.from, &s.recipient, &amount, &expiry),
        );
    }

    s.client.claim_transfer(&ids.get(0).unwrap(), &s.recipient);
    assert_eq!(s.client.total_funded(), 921);
    assert_eq!(s.client.total_released(), 17);
    assert_eq!(s.client.total_escrowed(), 904);
    assert_eq!(s.token_client().balance(&s.recipient), 17);
    assert_conserved(&s);

    s.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp = expiry + 1);
    s.client.cancel_transfer(&ids.get(1).unwrap(), &s.from);
    assert_eq!(s.client.total_released(), 220);
    assert_eq!(s.client.total_escrowed(), 701);
    assert_eq!(s.token_client().balance(&s.from), 1_282);
    assert_conserved(&s);

    s.client.sweep_expired(&ids.get(2).unwrap());
    assert_eq!(s.client.total_released(), 921);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.token_client().balance(&s.client.address), 0);
    assert_conserved(&s);
}

#[test]
fn batch_lifecycle_updates_conservation_per_successful_operation() {
    let s = setup();
    let second_sender = Address::generate(&s.env);
    StellarAssetClient::new(&s.env, &s.token).mint(&second_sender, &222);
    s.client.add_caller(&second_sender);
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let operations = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: second_sender,
            recipient: s.recipient.clone(),
            amount: 111,
            expiry,
        }),
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 222,
            expiry,
        }),
    ];

    let result = s.client.batch_operations(&operations);
    assert_eq!(result.len(), 2);
    assert_eq!(s.client.total_funded(), 333);
    assert_eq!(s.client.total_escrowed(), 333);
    assert_eq!(s.client.total_released(), 0);
    assert_conserved(&s);

    let claim = vec![
        &s.env,
        BatchOperation::Claim(ClaimTransferOperation {
            id: 1,
            recipient: s.recipient.clone(),
        }),
    ];
    s.client.batch_operations(&claim);
    assert_eq!(s.client.total_funded(), 333);
    assert_eq!(s.client.total_escrowed(), 222);
    assert_eq!(s.client.total_released(), 111);
    assert_conserved(&s);
}

#[test]
fn failed_claim_cannot_reduce_pending_or_increase_released_totals() {
    let s = setup();
    let id = s.create_default_transfer();
    let sender_before = s.token_client().balance(&s.from);
    let recipient_before = s.token_client().balance(&s.recipient);
    let escrow_before = s.token_client().balance(&s.client.address);

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_escrowed(&s.env, 0);
    });

    let result = s.client.try_claim_transfer(&id, &s.recipient);
    assert_eq!(result, Err(Ok(Error::AccountingOverflow)));
    assert_eq!(s.client.get_status(&id), Status::Pending);
    assert_eq!(s.client.total_funded(), DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_released(), 0);
    assert_balances_unchanged(&s, sender_before, recipient_before, escrow_before);
}

#[test]
fn failed_cancel_cannot_reduce_pending_or_increase_released_totals() {
    let s = setup();
    let expiry = s.future_expiry();
    let id = s
        .client
        .create_transfer(&s.from, &s.recipient, &275, &expiry);
    let sender_before = s.token_client().balance(&s.from);
    let escrow_before = s.token_client().balance(&s.client.address);
    s.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp = expiry + 1);

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_escrowed(&s.env, 100);
    });

    let result = s.client.try_cancel_transfer(&id, &s.from);
    assert_eq!(result, Err(Ok(Error::AccountingOverflow)));
    assert_eq!(s.client.get_status(&id), Status::Pending);
    assert_eq!(s.client.total_funded(), 275);
    assert_eq!(s.client.total_escrowed(), 100);
    assert_eq!(s.client.total_released(), 0);
    assert_eq!(s.token_client().balance(&s.from), sender_before);
    assert_eq!(s.token_client().balance(&s.client.address), escrow_before);
}

#[test]
fn failed_sweep_cannot_reduce_pending_or_increase_released_totals() {
    let s = setup();
    let id = s.create_default_transfer();
    s.env.ledger().with_mut(|ledger| {
        ledger.timestamp += DEFAULT_EXPIRY_OFFSET + 1;
    });
    let sender_before = s.token_client().balance(&s.from);
    let escrow_before = s.token_client().balance(&s.client.address);

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_escrowed(&s.env, 1);
    });

    let result = s.client.try_sweep_expired(&id);
    assert_eq!(result, Err(Ok(Error::AccountingOverflow)));
    assert_eq!(s.client.get_status(&id), Status::Pending);
    assert_eq!(s.client.total_funded(), DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(s.client.total_escrowed(), 1);
    assert_eq!(s.client.total_released(), 0);
    assert_eq!(s.token_client().balance(&s.from), sender_before);
    assert_eq!(s.token_client().balance(&s.client.address), escrow_before);
}

#[test]
fn funding_overflow_is_rejected_before_tokens_move() {
    let s = setup();
    let expiry = s.future_expiry();
    let sender_before = s.token_client().balance(&s.from);

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_funded(&s.env, i128::MAX);
    });

    let result = s
        .client
        .try_create_transfer(&s.from, &s.recipient, &100, &expiry);
    assert_eq!(result, Err(Ok(Error::AccountingOverflow)));
    assert_eq!(s.client.counter(), 0);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_released(), 0);
    assert_eq!(s.token_client().balance(&s.from), sender_before);
    assert_eq!(s.token_client().balance(&s.client.address), 0);
}

#[test]
fn released_overflow_is_rejected_without_changing_terminal_state() {
    let s = setup();
    let id = s.create_default_transfer();
    let recipient_before = s.token_client().balance(&s.recipient);

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_released(&s.env, i128::MAX);
    });

    let result = s.client.try_claim_transfer(&id, &s.recipient);
    assert_eq!(result, Err(Ok(Error::AccountingOverflow)));
    assert_eq!(s.client.get_status(&id), Status::Pending);
    assert_eq!(s.client.total_escrowed(), DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(s.client.total_released(), i128::MAX);
    assert_eq!(s.token_client().balance(&s.recipient), recipient_before);
    assert_eq!(
        s.token_client().balance(&s.client.address),
        DEFAULT_TRANSFER_AMOUNT
    );
}

#[test]
fn invariant_rejects_internal_conservation_mismatch() {
    let s = setup();
    s.create_default_transfer();

    s.env.as_contract(&s.client.address, || {
        crate::storage::set_total_released(&s.env, 1);
    });

    let result = s.client.try_check_supply_invariant();
    assert_eq!(result, Err(Ok(Error::SupplyInvariantViolation)));
}

#[test]
fn unsolicited_token_surplus_does_not_change_escrow_liability() {
    let s = setup();
    s.create_default_transfer();
    let token_admin = StellarAssetClient::new(&s.env, &s.token);
    token_admin.mint(&s.client.address, &999);

    assert_eq!(s.token_client().balance(&s.client.address), 1_399);
    assert_eq!(s.client.total_funded(), DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(s.client.total_escrowed(), DEFAULT_TRANSFER_AMOUNT);
    assert_eq!(s.client.total_released(), 0);
    assert_conserved(&s);
}

#[test]
fn new_accounting_totals_start_at_zero_after_initialization() {
    let s = setup();

    assert_eq!(s.client.total_funded(), 0);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_released(), 0);
    assert_conserved(&s);
}

#[test]
fn cap_validation_still_uses_pending_total_not_lifetime_total() {
    let s = setup();
    let token_admin = StellarAssetClient::new(&s.env, &s.token);
    token_admin.mint(&s.from, &MAX_TOTAL_ESCROWED);
    let expiry = s.future_expiry();

    let id = s
        .client
        .create_transfer(&s.from, &s.recipient, &MAX_TOTAL_ESCROWED, &expiry);
    assert_eq!(id, 1);
    assert_eq!(s.client.total_funded(), MAX_TOTAL_ESCROWED);
    assert_eq!(s.client.total_escrowed(), MAX_TOTAL_ESCROWED);
    assert_conserved(&s);

    s.env
        .ledger()
        .with_mut(|ledger| ledger.timestamp = expiry + 1);
    s.client.sweep_expired(&id);
    assert_eq!(s.client.total_funded(), MAX_TOTAL_ESCROWED);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_released(), MAX_TOTAL_ESCROWED);
    assert_conserved(&s);
}
