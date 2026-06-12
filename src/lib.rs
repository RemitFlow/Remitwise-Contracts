#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

mod error;
mod events;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::error::Error;

/// The RemitFlow remittance escrow contract.
#[contract]
pub struct RemitFlowContract;

#[contractimpl]
impl RemitFlowContract {
    /// Initialize the contract with an administrator and token address.
    ///
    /// Can only be called once; subsequent calls return
    /// [`Error::AlreadyInitialized`].
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_counter(&env, 0);
        storage::extend_instance(&env);
        events::init(&env, &admin, &token);
        Ok(())
    }
}
