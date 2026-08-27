#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

use crate::{Error, RemitFlowContract, RemitFlowContractClient};

fn setup() -> (Env, RemitFlowContractClient<'static>, Address, BytesN<32>) {
    let env = Env::default();
    let contract = env.register_contract(None, RemitFlowContract);
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let client = RemitFlowContractClient::new(&env, &contract);
    env.mock_all_auths();
    client.initialize(&admin, &token);
    let baseline = BytesN::from_array(&env, &[1; 32]);
    (env, client, admin, baseline)
}

#[test]
fn baseline_is_visible_and_starts_at_zero() {
    let (_env, client, _admin, baseline) = setup();
    client.set_upgrade_baseline(&baseline);
    assert_eq!(client.get_upgrade_artifact(), baseline);
    assert_eq!(client.get_upgrade_version(), 0);
}

#[test]
fn baseline_can_only_be_registered_once() {
    let (_env, client, _admin, baseline) = setup();
    client.set_upgrade_baseline(&baseline);
    assert_eq!(
        client.try_set_upgrade_baseline(&BytesN::from_array(&_env, &[2; 32])),
        Err(Ok(Error::UpgradeBaselineAlreadySet))
    );
}

#[test]
fn upgrade_requires_a_recorded_baseline() {
    let (env, client, _admin, baseline) = setup();
    let replacement = BytesN::from_array(&env, &[2; 32]);
    assert_eq!(
        client.try_upgrade(&baseline, &replacement, &1),
        Err(Ok(Error::UpgradeArtifactMismatch))
    );
}

#[test]
fn wrong_expected_artifact_is_rejected_before_mutation() {
    let (env, client, _admin, baseline) = setup();
    client.set_upgrade_baseline(&baseline);
    let wrong = BytesN::from_array(&env, &[9; 32]);
    let replacement = BytesN::from_array(&env, &[2; 32]);
    assert_eq!(
        client.try_upgrade(&wrong, &replacement, &1),
        Err(Ok(Error::UpgradeArtifactMismatch))
    );
    assert_eq!(client.get_upgrade_artifact(), baseline);
    assert_eq!(client.get_upgrade_version(), 0);
}

#[test]
fn unchanged_artifact_is_rejected() {
    let (_env, client, _admin, baseline) = setup();
    client.set_upgrade_baseline(&baseline);
    assert_eq!(
        client.try_upgrade(&baseline, &baseline, &1),
        Err(Ok(Error::UpgradeArtifactUnchanged))
    );
}

#[test]
fn release_numbers_must_be_sequential() {
    let (env, client, _admin, baseline) = setup();
    client.set_upgrade_baseline(&baseline);
    let replacement = BytesN::from_array(&env, &[2; 32]);
    assert_eq!(
        client.try_upgrade(&baseline, &replacement, &2),
        Err(Ok(Error::UpgradeVersionInvalid))
    );
    assert_eq!(client.get_upgrade_version(), 0);
}

#[test]
fn uninitialized_upgrade_state_is_not_exposed_as_a_valid_release() {
    let env = Env::default();
    let contract = env.register_contract(None, RemitFlowContract);
    let client = RemitFlowContractClient::new(&env, &contract);
    assert_eq!(client.get_upgrade_version(), 0);
    assert_eq!(
        client.try_get_upgrade_artifact(),
        Err(Ok(Error::NotInitialized))
    );
}
