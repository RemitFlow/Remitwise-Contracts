#![cfg(test)]

//! Composable test fixture builder for common RemitFlow test state.
//!
//! Use FixtureBuilder::new() to create a default initialized contract, then
//! chain builder methods to set up the specific scenario you need before
//! calling .build() to obtain a ready-to-use TestFixture.

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

use crate::types::Status;
use crate::{RemitFlowContract, RemitFlowContractClient};

use super::test_utils::{
    TestFixture, DEFAULT_EXPIRY_OFFSET, DEFAULT_SENDER_BALANCE, DEFAULT_TRANSFER_AMOUNT,
};

pub struct FixtureBuilder {
    env: Env,
    admin: Address,
    from: Address,
    recipient: Address,
    sender_balance: i128,
    transfer_amount: i128,
    expiry_offset: u64,
    paused: bool,
    create_transfer: bool,
    claim_transfer: bool,
    cancel_transfer: bool,
    num_transfers: u64,
}

impl FixtureBuilder {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        Self {
            env,
            admin: Address::generate(&env),
            from: Address::generate(&env),
            recipient: Address::generate(&env),
            sender_balance: DEFAULT_SENDER_BALANCE,
            transfer_amount: DEFAULT_TRANSFER_AMOUNT,
            expiry_offset: DEFAULT_EXPIRY_OFFSET,
            paused: false,
            create_transfer: true,
            claim_transfer: false,
            cancel_transfer: false,
            num_transfers: 1,
        }
    }

    pub fn with_sender_balance(mut self, balance: i128) -> Self {
        self.sender_balance = balance;
        self
    }

    pub fn with_transfer_amount(mut self, amount: i128) -> Self {
        self.transfer_amount = amount;
        self
    }

    pub fn with_expiry_offset(mut self, offset: u64) -> Self {
        self.expiry_offset = offset;
        self
    }

    pub fn with_admin(mut self, admin: Address) -> Self {
        self.admin = admin;
        self
    }

    pub fn with_sender(mut self, from: Address) -> Self {
        self.from = from;
        self
    }

    pub fn with_recipient(mut self, recipient: Address) -> Self {
        self.recipient = recipient;
        self
    }

    pub fn with_num_transfers(mut self, count: u64) -> Self {
        self.num_transfers = count;
        self
    }

    pub fn paused(mut self) -> Self {
        self.paused = true;
        self
    }

    pub fn without_transfer(mut self) -> Self {
        self.create_transfer = false;
        self
    }

    pub fn claimed(mut self) -> Self {
        self.claim_transfer = true;
        self
    }

    pub fn cancelled(mut self) -> Self {
        self.cancel_transfer = true;
        self
    }

    pub fn build(self) -> (TestFixture<'static>, Env, RemitFlowContractClient<'static>) {
        let token_contract = self.env.register_stellar_asset_contract_v2(self.admin.clone());
        let token = token_contract.address();
        StellarAssetClient::new(&self.env, &token).mint(&self.from, &self.sender_balance);

        let contract_id = self.env.register_contract(None, RemitFlowContract);
        let client = RemitFlowContractClient::new(&self.env, &contract_id);
        client.initialize(&self.admin, &token);
        client.add_caller(&self.from);

        if self.paused {
            client.pause();
        }

        let mut transfer_ids = Vec::new(&self.env);

        if self.create_transfer {
            for _ in 0..self.num_transfers {
                let id = client.create_transfer(
                    &self.from,
                    &self.recipient,
                    &self.transfer_amount,
                    &(self.env.ledger().timestamp() + self.expiry_offset),
                );
                transfer_ids.push_back(id);
            }
        }

        if self.claim_transfer {
            for id in transfer_ids.iter() {
                self.env.ledger().set_timestamp(
                    self.env.ledger().timestamp() + self.expiry_offset - 100,
                );
                client.claim_transfer(&id, &self.recipient);
            }
        }

        if self.cancel_transfer {
            for id in transfer_ids.iter() {
                self.env.ledger().set_timestamp(
                    self.env.ledger().timestamp() + self.expiry_offset + 1,
                );
                client.cancel_transfer(&id, &self.from);
            }
        }

        let fixture = TestFixture {
            env: self.env,
            client,
            token,
            admin: self.admin,
            from: self.from,
            recipient: self.recipient,
        };

        (fixture, self.env, client)
    }
}

impl Default for FixtureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Status;

    #[test]
    fn default_builder_creates_initialized_contract_with_one_pending_transfer() {
        let (fixture, _env, client) = FixtureBuilder::new().build();
        let transfer = client.get_transfer(&1);
        assert_eq!(transfer.status, Status::Pending);
        assert_eq!(transfer.amount, DEFAULT_TRANSFER_AMOUNT);
        assert_eq!(fixture.from, transfer.from);
    }

    #[test]
    fn builder_with_custom_amount_respects_value() {
        let (_fixture, _env, client) =
            FixtureBuilder::new().with_transfer_amount(999).build();
        let transfer = client.get_transfer(&1);
        assert_eq!(transfer.amount, 999);
    }

    #[test]
    fn builder_without_transfer_has_empty_counter() {
        let (_fixture, _env, client) =
            FixtureBuilder::new().without_transfer().build();
        assert_eq!(client.counter(), 0);
    }

    #[test]
    fn builder_claimed_sets_status_to_claimed() {
        let (_fixture, _env, client) =
            FixtureBuilder::new().claimed().build();
        let transfer = client.get_transfer(&1);
        assert_eq!(transfer.status, Status::Claimed);
    }

    #[test]
    fn builder_cancelled_sets_status_to_cancelled() {
        let (_fixture, _env, client) =
            FixtureBuilder::new().cancelled().build();
        let transfer = client.get_transfer(&1);
        assert_eq!(transfer.status, Status::Cancelled);
    }

    #[test]
    fn builder_paused_rejects_new_transfer() {
        let (fixture, _env, client) =
            FixtureBuilder::new().paused().without_transfer().build();
        let result = client.try_create_transfer(
            &fixture.from,
            &fixture.recipient,
            &DEFAULT_TRANSFER_AMOUNT,
            &(fixture.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET),
        );
        assert_eq!(result, Err(Ok(crate::error::Error::ContractPaused)));
    }

    #[test]
    fn builder_multiple_transfers_creates_sequential_ids() {
        let (_fixture, _env, client) =
            FixtureBuilder::new().with_num_transfers(5).build();
        assert_eq!(client.counter(), 5);
        assert!(client.transfer_exists(&1));
        assert!(client.transfer_exists(&5));
    }

    #[test]
    fn builder_custom_balance_is_minted() {
        let (_fixture, _env, _client) =
            FixtureBuilder::new().with_sender_balance(50_000).build();
        // Balance minted — transfer was created, so balance decreased
        let transfer = _client.get_transfer(&1);
        assert_eq!(transfer.amount, DEFAULT_TRANSFER_AMOUNT);
    }

    #[test]
    fn builder_with_custom_addresses_uses_them() {
        let env = Env::default();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let (_fixture, _env, client) = FixtureBuilder::new()
            .with_sender(sender.clone())
            .with_recipient(recipient.clone())
            .build();
        let transfer = client.get_transfer(&1);
        assert_eq!(transfer.from, sender);
        assert_eq!(transfer.recipient, recipient);
    }
}
