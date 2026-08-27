#![cfg(test)]

//! Regression coverage for atomic, deterministic, idempotent batch execution.

use soroban_sdk::testutils::storage::Persistent as _;
use soroban_sdk::vec;

use crate::test_utils::{TestFixture, DEFAULT_EXPIRY_OFFSET};
use crate::types::{
    BatchOperation, BatchOperationResult, ClaimTransferOperation, CreateTransferOperation, Status,
};
use crate::{Error, MAX_BATCH_SIZE};

fn setup() -> TestFixture<'static> {
    TestFixture::new()
}

fn create_and_claim_batch(s: &TestFixture<'_>) -> soroban_sdk::Vec<BatchOperation> {
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 275,
            expiry,
        }),
        BatchOperation::Claim(ClaimTransferOperation {
            id: 1,
            recipient: s.recipient.clone(),
        }),
    ]
}

#[test]
fn idempotent_batch_returns_stable_results_without_reapplying_items() {
    let s = setup();
    let operations = create_and_claim_batch(&s);

    let first = s.client.batch_operations_idempotent(&41, &operations);
    assert_eq!(
        first,
        vec![
            &s.env,
            BatchOperationResult::Created(1),
            BatchOperationResult::Claimed,
        ]
    );
    let counter_after_first = s.client.counter();
    let sender_after_first = s.token_client().balance(&s.from);
    let recipient_after_first = s.token_client().balance(&s.recipient);

    let retry = s.client.batch_operations_idempotent(&41, &operations);
    assert_eq!(retry, first);
    assert_eq!(s.client.counter(), counter_after_first);
    assert_eq!(s.token_client().balance(&s.from), sender_after_first);
    assert_eq!(
        s.token_client().balance(&s.recipient),
        recipient_after_first
    );
    assert_eq!(s.client.get_status(&1), Status::Claimed);
}

#[test]
fn idempotency_key_conflict_is_rejected_without_state_change() {
    let s = setup();
    let first_operations = create_and_claim_batch(&s);
    s.client.batch_operations_idempotent(&99, &first_operations);

    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let different_operations = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 1,
            expiry,
        }),
    ];
    let sender_before = s.token_client().balance(&s.from);
    let counter_before = s.client.counter();

    let result = s
        .client
        .try_batch_operations_idempotent(&99, &different_operations);
    assert_eq!(result, Err(Ok(Error::BatchIdConflict)));
    assert_eq!(s.client.counter(), counter_before);
    assert_eq!(s.token_client().balance(&s.from), sender_before);
    assert_eq!(s.client.total_escrowed(), 0);
}

#[test]
fn zero_id_is_rejected_before_any_batch_validation_or_execution() {
    let s = setup();
    let operations = create_and_claim_batch(&s);

    let result = s.client.try_batch_operations_idempotent(&0, &operations);
    assert_eq!(result, Err(Ok(Error::InvalidBatchId)));
    assert_eq!(s.client.counter(), 0);
    assert!(!s.client.transfer_exists(&1));
    assert_eq!(s.token_client().balance(&s.from), 1_000);
    assert_eq!(s.token_client().balance(&s.client.address), 0);
}

#[test]
fn failed_idempotent_batch_does_not_reserve_its_id() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let invalid = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 200,
            expiry,
        }),
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.from.clone(),
            amount: 1,
            expiry,
        }),
    ];

    let failed = s.client.try_batch_operations_idempotent(&7, &invalid);
    assert_eq!(failed, Err(Ok(Error::SameParty)));
    assert_eq!(s.client.counter(), 0);
    assert_eq!(s.client.total_escrowed(), 0);

    let valid = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 200,
            expiry,
        }),
    ];
    let result = s.client.batch_operations_idempotent(&7, &valid);
    assert_eq!(result, vec![&s.env, BatchOperationResult::Created(1)]);
    assert_eq!(s.client.total_escrowed(), 200);
}

#[test]
fn idempotent_batch_preserves_indexed_order_for_mixed_operations() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let operations = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 100,
            expiry,
        }),
        BatchOperation::Claim(ClaimTransferOperation {
            id: 1,
            recipient: s.recipient.clone(),
        }),
    ];

    let result = s.client.batch_operations_idempotent(&8, &operations);
    assert_eq!(result.get(0), Some(BatchOperationResult::Created(1)));
    assert_eq!(result.get(1), Some(BatchOperationResult::Claimed));
    assert_eq!(result.len(), operations.len());
}

#[test]
fn oversized_idempotent_batch_is_rejected_without_receipt() {
    let s = setup();
    let mut operations = vec![&s.env];
    for _ in 0..=MAX_BATCH_SIZE {
        operations.push_back(BatchOperation::Claim(ClaimTransferOperation {
            id: 1,
            recipient: s.recipient.clone(),
        }));
    }

    let result = s.client.try_batch_operations_idempotent(&10, &operations);
    assert_eq!(result, Err(Ok(Error::BatchTooLarge)));
    s.env.as_contract(&s.client.address, || {
        assert!(!s
            .env
            .storage()
            .persistent()
            .has(&crate::storage::PersistentKey::Batch(10)));
    });
}

#[test]
fn distinct_idempotency_keys_can_execute_distinct_batches() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let first = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 100,
            expiry,
        }),
    ];
    let second = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 200,
            expiry,
        }),
    ];

    assert_eq!(
        s.client.batch_operations_idempotent(&1, &first),
        vec![&s.env, BatchOperationResult::Created(1)]
    );
    assert_eq!(
        s.client.batch_operations_idempotent(&2, &second),
        vec![&s.env, BatchOperationResult::Created(2)]
    );
    assert_eq!(s.client.total_escrowed(), 300);
}

#[test]
fn idempotency_receipt_gets_persistent_ttl() {
    let s = setup();
    let operations = create_and_claim_batch(&s);
    s.client.batch_operations_idempotent(&12, &operations);

    s.env.as_contract(&s.client.address, || {
        let key = crate::storage::PersistentKey::Batch(12);
        assert!(s.env.storage().persistent().has(&key));
        assert_eq!(
            s.env.storage().persistent().get_ttl(&key),
            crate::storage::PERSISTENT_BUMP_AMOUNT
        );
    });
}

#[test]
fn legacy_batch_entrypoint_remains_atomic_and_unchanged() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let operations = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 150,
            expiry,
        }),
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.from.clone(),
            amount: 250,
            expiry,
        }),
    ];

    let result = s.client.try_batch_operations(&operations);
    assert_eq!(result, Err(Ok(Error::SameParty)));
    assert_eq!(s.client.counter(), 0);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.client.total_escrowed(), 0);
    assert_eq!(s.token_client().balance(&s.from), 1_000);
}

#[test]
fn retry_after_atomic_failure_can_use_same_id_with_same_payload() {
    let s = setup();
    let expiry = s.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET;
    let operations = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 75,
            expiry,
        }),
        BatchOperation::Claim(ClaimTransferOperation {
            id: 999,
            recipient: s.recipient.clone(),
        }),
    ];

    let failed = s.client.try_batch_operations_idempotent(&13, &operations);
    assert_eq!(failed, Err(Ok(Error::TransferNotFound)));
    assert_eq!(s.client.counter(), 0);

    let valid = vec![
        &s.env,
        BatchOperation::Create(CreateTransferOperation {
            from: s.from.clone(),
            recipient: s.recipient.clone(),
            amount: 75,
            expiry,
        }),
    ];
    s.client.batch_operations_idempotent(&13, &valid);
    assert_eq!(s.client.counter(), 1);
    assert_eq!(s.client.total_escrowed(), 75);
}
