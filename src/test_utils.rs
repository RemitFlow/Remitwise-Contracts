#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

use crate::{RemitFlowContract, RemitFlowContractClient};

pub(crate) const DEFAULT_SENDER_BALANCE: i128 = 1_000;
pub(crate) const DEFAULT_TRANSFER_AMOUNT: i128 = 400;
pub(crate) const DEFAULT_EXPIRY_OFFSET: u64 = 1_000;

/// Common fixture for contract tests.
///
/// The fixture deploys and initializes RemitFlow, deploys its Stellar Asset
/// Contract, creates the standard test actors, and funds the sender.
pub(crate) struct TestFixture<'a> {
    pub(crate) env: Env,
    pub(crate) client: RemitFlowContractClient<'a>,
    pub(crate) token: Address,
    pub(crate) admin: Address,
    pub(crate) from: Address,
    pub(crate) recipient: Address,
}

impl<'a> TestFixture<'a> {
    pub(crate) fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let from = Address::generate(&env);
        let recipient = Address::generate(&env);

        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_contract.address();
        StellarAssetClient::new(&env, &token).mint(&from, &DEFAULT_SENDER_BALANCE);

        let contract_id = env.register(RemitFlowContract, ());
        let client = RemitFlowContractClient::new(&env, &contract_id);
        client.initialize(&admin, &token);

        Self {
            env,
            client,
            token,
            admin,
            from,
            recipient,
        }
    }

    pub(crate) fn token_client(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.token)
    }

    pub(crate) fn future_expiry(&self) -> u64 {
        self.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET
    }

    pub(crate) fn create_default_transfer(&self) -> u64 {
        self.client.create_transfer(
            &self.from,
            &self.recipient,
            &DEFAULT_TRANSFER_AMOUNT,
            &self.future_expiry(),
        )
    }
}
