#![cfg(test)]

//! Contract-level fixtures for the versioned event payload contract.

use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

use crate::events::{
    self, ActorEvent, AdminTransferEvent, CancelledEvent, ClaimedEvent, CreatedEvent, InitEvent,
    EVENT_SCHEMA_VERSION,
};

fn latest_data(env: &Env) -> soroban_sdk::Val {
    env.events().all().last().unwrap().2.clone()
}

#[test]
fn init_payload_is_versioned_and_identifies_token() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    env.as_contract(&contract_id, || events::init(&env, &admin, &token));

    let payload: InitEvent = latest_data(&env).into_val(&env);
    assert_eq!(payload.metadata.schema_version, 1);
    assert_eq!(
        payload.metadata.amount_unit,
        soroban_sdk::String::from_str(&env, "")
    );
    assert_eq!(
        payload.metadata.timestamp_unit,
        soroban_sdk::String::from_str(&env, "ledger_seconds")
    );
    assert_eq!(payload.admin, admin);
    assert_eq!(payload.token, token);
}

#[test]
fn created_payload_contains_explicit_units_and_all_transfer_fields() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);
    env.as_contract(&contract_id, || {
        events::created(&env, 7, &from, &recipient, 123_456, 9_999)
    });

    let payload: CreatedEvent = latest_data(&env).into_val(&env);
    assert_eq!(payload.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(
        payload.metadata.amount_unit,
        soroban_sdk::String::from_str(&env, "token_base_units")
    );
    assert_eq!(
        payload.metadata.timestamp_unit,
        soroban_sdk::String::from_str(&env, "ledger_seconds")
    );
    assert_eq!(payload.transfer_id, 7);
    assert_eq!(payload.from, from);
    assert_eq!(payload.recipient, recipient);
    assert_eq!(payload.amount, 123_456);
    assert_eq!(payload.expiry, 9_999);
}

#[test]
fn claim_and_cancel_payloads_keep_transfer_identity() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let actor = Address::generate(&env);
    env.as_contract(&contract_id, || events::claimed(&env, 12, &actor, 50));
    let claimed: ClaimedEvent = latest_data(&env).into_val(&env);
    assert_eq!(claimed.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(claimed.transfer_id, 12);
    assert_eq!(claimed.recipient, actor);
    assert_eq!(claimed.amount, 50);

    env.as_contract(&contract_id, || events::cancelled(&env, 13, &actor, 75));
    let cancelled: CancelledEvent = latest_data(&env).into_val(&env);
    assert_eq!(cancelled.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(cancelled.transfer_id, 13);
    assert_eq!(cancelled.from, actor);
    assert_eq!(cancelled.amount, 75);
}

#[test]
fn administrative_payloads_have_a_common_actor_shape() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let actor = Address::generate(&env);
    env.as_contract(&contract_id, || events::paused(&env, &actor));
    let paused: ActorEvent = latest_data(&env).into_val(&env);
    assert_eq!(paused.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(paused.actor, actor);

    env.as_contract(&contract_id, || events::unpaused(&env, &actor));
    let unpaused: ActorEvent = latest_data(&env).into_val(&env);
    assert_eq!(unpaused.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(unpaused.actor, actor);

    env.as_contract(&contract_id, || events::caller_added(&env, &actor));
    let added: ActorEvent = latest_data(&env).into_val(&env);
    assert_eq!(added.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(added.actor, actor);

    env.as_contract(&contract_id, || events::caller_removed(&env, &actor));
    let removed: ActorEvent = latest_data(&env).into_val(&env);
    assert_eq!(removed.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(removed.actor, actor);
}

#[test]
fn admin_transfer_payload_distinguishes_old_and_new_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let old_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    env.as_contract(&contract_id, || {
        events::admin_transfer_started(&env, &old_admin, &new_admin)
    });
    let started: AdminTransferEvent = latest_data(&env).into_val(&env);
    assert_eq!(started.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(started.old_admin, old_admin);
    assert_eq!(started.new_admin, new_admin);

    env.as_contract(&contract_id, || {
        events::admin_transfer_completed(&env, &old_admin, &new_admin)
    });
    let completed: AdminTransferEvent = latest_data(&env).into_val(&env);
    assert_eq!(completed.metadata.schema_version, EVENT_SCHEMA_VERSION);
    assert_eq!(completed.old_admin, old_admin);
    assert_eq!(completed.new_admin, new_admin);
}

#[test]
fn topics_remain_stable_while_payloads_gain_schema_metadata() {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::RemitFlowContract);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);
    env.as_contract(&contract_id, || {
        events::created(&env, 42, &from, &recipient, 1, 2)
    });
    let event = env.events().all().last().unwrap();
    let topic: Symbol = event.1.get(0).unwrap().into_val(&env);
    let id: u64 = event.1.get(1).unwrap().into_val(&env);
    assert_eq!(topic, Symbol::new(&env, "created"));
    assert_eq!(id, 42);
    assert_eq!(event.1.len(), 2);
}
