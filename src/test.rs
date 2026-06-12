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
