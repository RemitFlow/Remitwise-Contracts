#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

// soroban #[contracttype] generates Arbitrary impls under 	estutils,
// which need std. Link it for test builds only; wasm builds stay no_std.
#[cfg(test)]
extern crate std;

mod error;
mod events;
pub mod math;
mod storage;
mod types;

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod test;
mod test_utils;

use soroban_sdk::{contract, contractimpl, contractmeta, token, Address, Env, Vec};

use crate::error::Error;
use crate::types::{BatchOperation, BatchOperationResult, ConfiguredLimits, Status, Transfer};

contractmeta!(key = "name", val = "RemitFlow");
contractmeta!(key = "version", val = "0.1.0");
contractmeta!(
    key = "description",
    val = "Cross-border remittance escrow for Soroban/Stellar"
);

/// Largest token amount accepted for a single escrowed transfer.
pub const MAX_AMOUNT: i128 = 1_000_000_000_000_000_000;

/// Maximum allowed distance, in seconds, between now and a transfer's expiry.
pub const MAX_EXPIRY_WINDOW: u64 = 31_536_000;

/// Global cap on the total escrowed amount.
pub const MAX_ACCOUNT_OPS: u32 = 10_000;

pub const MAX_TOTAL_ESCROWED: i128 = MAX_AMOUNT;

/// Minimum cooldown in seconds between privileged administrative calls.
pub const PRIVILEGED_COOLDOWN: u64 = 300;

/// Maximum number of records returned by a paginated transfer query.
pub const MAX_PAGE_SIZE: u32 = 100;

fn require_external_address(env: &Env, address: &Address) -> Result<(), Error> {
    if *address == env.current_contract_address() {
        return Err(Error::InvalidAddress);
    }
    Ok(())
}

/// Reject the call if a privileged operation was executed less than
/// [PRIVILEGED_COOLDOWN] seconds ago.
fn require_cooldown(env: &Env) -> Result<(), Error> {
    let last = storage::get_last_privileged_call(env);
    if last > 0 {
        let now = env.ledger().timestamp();
        if now.saturating_sub(last) < PRIVILEGED_COOLDOWN {
            return Err(Error::CooldownNotElapsed);
        }
    }
    Ok(())
}

/// Record that a privileged call just executed at the current ledger time.
fn record_privileged_call(env: &Env) {
    storage::set_last_privileged_call(env, env.ledger().timestamp());
}

fn assert_supply_invariant(env: &Env, token: &Address) -> Result<(), Error> {
    let balance = token::Client::new(env, token).balance(&env.current_contract_address());
    if balance < storage::get_total_escrowed(env) {
        return Err(Error::SupplyInvariantViolation);
    }
    Ok(())
}

/// The RemitFlow remittance escrow contract.
#[contract]
pub struct RemitFlowContract;

#[contractimpl]
impl RemitFlowContract {
    pub fn batch_operations(
        env: Env,
        operations: Vec<BatchOperation>,
    ) -> Result<Vec<BatchOperationResult>, Error> {
        let mut results = Vec::new(&env);
        for operation in operations.iter() {
            let result = match operation {
                BatchOperation::Create(params) => {
                    let id = Self::create_transfer(
                        env.clone(),
                        params.from,
                        params.recipient,
                        params.amount,
                        params.expiry,
                    )?;
                    BatchOperationResult::Created(id)
                },
                BatchOperation::Claim(params) => {
                    Self::claim_transfer(env.clone(), params.id, params.recipient)?;
                    BatchOperationResult::Claimed
                },
                BatchOperation::Cancel(params) => {
                    Self::cancel_transfer(env.clone(), params.id, params.from)?;
                    BatchOperationResult::Cancelled
                },
            };
            results.push_back(result);
        }
        Ok(results)
    }

    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        require_external_address(&env, &admin)?;
        require_external_address(&env, &token)?;
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_counter(&env, 0);
        storage::set_initialized_at(&env, env.ledger().timestamp());
        storage::extend_instance(&env);
        events::init(&env, &admin, &token);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    pub fn get_token(env: Env) -> Result<Address, Error> {
        storage::get_token(&env).ok_or(Error::NotInitialized)
    }

    pub fn get_initialized_at(env: Env) -> Result<u64, Error> {
        storage::get_initialized_at(&env).ok_or(Error::NotInitialized)
    }

    pub fn get_balances(env: Env, addresses: Vec<Address>) -> Result<Vec<i128>, Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        let client = token::Client::new(&env, &token);
        let mut balances = Vec::new(&env);
        for address in addresses.iter() {
            balances.push_back(client.balance(&address));
        }
        Ok(balances)
    }

    pub fn counter(env: Env) -> u64 {
        storage::get_counter(&env)
    }

    pub fn pause(env: Env) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_paused(&env, true);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_paused(&env, false);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::unpaused(&env, &admin);
        Ok(())
    }

    pub fn create_transfer(
        env: Env,
        from: Address,
        recipient: Address,
        amount: i128,
        expiry: u64,
    ) -> Result<u64, Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        require_external_address(&env, &from)?;
        require_external_address(&env, &recipient)?;
        if storage::get_paused(&env) {
            return Err(Error::ContractPaused);
        }
        if !storage::is_caller_allowed(&env, &from) {
            return Err(Error::CallerNotAllowed);
        }
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if storage::get_account_op_count(&env, &from) >= MAX_ACCOUNT_OPS {
            return Err(Error::AccountLimitReached);
        }
        if amount > MAX_AMOUNT {
            return Err(Error::AmountTooLarge);
        }
        let total_escrowed = storage::get_total_escrowed(&env);
        let updated_total =
            math::checked_add_amount(total_escrowed, amount).ok_or(Error::AmountTooLarge)?;
        if updated_total > MAX_TOTAL_ESCROWED {
            return Err(Error::EscrowCapReached);
        }
        let now = env.ledger().timestamp();
        if expiry <= now {
            return Err(Error::InvalidExpiry);
        }
        if expiry - now > MAX_EXPIRY_WINDOW {
            return Err(Error::ExpiryTooFar);
        }
        if from == recipient {
            return Err(Error::SameParty);
        }
        from.require_auth();

        let id =
            math::checked_increment(storage::get_counter(&env)).ok_or(Error::CounterOverflow)?;

        token::Client::new(&env, &token).transfer(&from, &env.current_contract_address(), &amount);

        let transfer = Transfer {
            id,
            from: from.clone(),
            recipient: recipient.clone(),
            amount,
            expiry,
            status: Status::Pending,
        };
        storage::set_transfer(&env, &transfer);
        storage::set_counter(&env, id);
        storage::set_total_escrowed(&env, updated_total);
        storage::increment_account_op_count(&env, &from);
        assert_supply_invariant(&env, &token)?;
        storage::extend_instance(&env);
        events::created(&env, id, &from, &recipient, amount, expiry);
        Ok(id)
    }

    pub fn claim_transfer(env: Env, id: u64, recipient: Address) -> Result<(), Error> {
        require_external_address(&env, &recipient)?;
        let mut transfer = storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)?;
        if transfer.recipient != recipient {
            return Err(Error::Unauthorized);
        }
        if transfer.status != Status::Pending {
            return Err(Error::NotPending);
        }
        if env.ledger().timestamp() > transfer.expiry {
            return Err(Error::Expired);
        }
        recipient.require_auth();

        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &recipient,
            &transfer.amount,
        );

        transfer.status = Status::Claimed;
        storage::set_total_escrowed(
            &env,
            storage::get_total_escrowed(&env).saturating_sub(transfer.amount),
        );
        assert_supply_invariant(&env, &token)?;
        let amount = transfer.amount;
        storage::set_transfer(&env, &transfer);
        storage::extend_instance(&env);
        events::claimed(&env, id, &recipient, amount);
        Ok(())
    }

    pub fn cancel_transfer(env: Env, id: u64, from: Address) -> Result<(), Error> {
        require_external_address(&env, &from)?;
        let mut transfer = storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)?;
        if transfer.from != from {
            return Err(Error::Unauthorized);
        }
        if transfer.status != Status::Pending {
            return Err(Error::NotPending);
        }
        if env.ledger().timestamp() <= transfer.expiry {
            return Err(Error::NotExpired);
        }
        from.require_auth();

        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &from,
            &transfer.amount,
        );

        transfer.status = Status::Cancelled;
        storage::set_total_escrowed(
            &env,
            storage::get_total_escrowed(&env).saturating_sub(transfer.amount),
        );
        assert_supply_invariant(&env, &token)?;
        let amount = transfer.amount;
        storage::set_transfer(&env, &transfer);
        storage::extend_instance(&env);
        events::cancelled(&env, id, &from, amount);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::get_paused(&env)
    }

    pub fn get_transfer(env: Env, id: u64) -> Result<Transfer, Error> {
        storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)
    }

    pub fn transfer_exists(env: Env, id: u64) -> bool {
        storage::has_transfer(&env, id)
    }

    pub fn get_status(env: Env, id: u64) -> Result<Status, Error> {
        storage::get_transfer(&env, id)
            .map(|transfer| transfer.status)
            .ok_or(Error::TransferNotFound)
    }

    pub fn get_transfers_paged(env: Env, start_id: u64, limit: u32) -> Vec<Transfer> {
        let last = storage::get_counter(&env);
        let mut page = Vec::new(&env);
        let mut id = start_id.max(1);
        let page_size = limit.min(MAX_PAGE_SIZE);
        while id <= last && page.len() < page_size {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                page.push_back(transfer);
            }
            match id.checked_add(1) {
                Some(next_id) => id = next_id,
                None => break,
            }
        }
        page
    }

    pub fn total_escrowed(env: Env) -> i128 {
        let last = storage::get_counter(&env);
        let mut total: i128 = 0;
        let mut id = 1u64;
        while id <= last {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                if transfer.status == Status::Pending {
                    total = math::saturating_add_amount(total, transfer.amount);
                }
            }
            id += 1;
        }
        total
    }

    pub fn check_supply_invariant(env: Env) -> Result<(), Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        assert_supply_invariant(&env, &token)
    }

    pub fn is_expired(env: Env, id: u64) -> Result<bool, Error> {
        let transfer = storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)?;
        Ok(env.ledger().timestamp() > transfer.expiry)
    }

    pub fn count_for_sender(env: Env, from: Address) -> u64 {
        let last = storage::get_counter(&env);
        let mut count = 0u64;
        let mut id = 1u64;
        while id <= last {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                if transfer.from == from {
                    count = math::saturating_add_with_cap(count, 1, u64::MAX);
                }
            }
            id += 1;
        }
        count
    }

    pub fn count_for_recipient(env: Env, recipient: Address) -> u64 {
        let last = storage::get_counter(&env);
        let mut count = 0u64;
        let mut id = 1u64;
        while id <= last {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                if transfer.recipient == recipient {
                    count = math::saturating_add_with_cap(count, 1, u64::MAX);
                }
            }
            id += 1;
        }
        count
    }

    pub fn count_by_status(env: Env, status: Status) -> u64 {
        let last = storage::get_counter(&env);
        let mut count = 0u64;
        let mut id = 1u64;
        while id <= last {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                if transfer.status == status {
                    count = math::saturating_add_with_cap(count, 1, u64::MAX);
                }
            }
            id += 1;
        }
        count
    }

    pub fn add_caller(env: Env, caller: Address) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        require_external_address(&env, &caller)?;
        storage::set_caller_allowed(&env, &caller, true);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::caller_added(&env, &caller);
        Ok(())
    }

    pub fn remove_caller(env: Env, caller: Address) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_caller_allowed(&env, &caller, false);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::caller_removed(&env, &caller);
        Ok(())
    }

    pub fn is_caller_allowed(env: Env, caller: Address) -> bool {
        storage::is_caller_allowed(&env, &caller)
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        require_external_address(&env, &new_admin)?;
        storage::set_pending_admin(&env, &new_admin);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::admin_transfer_started(&env, &admin, &new_admin);
        Ok(())
    }

    pub fn accept_admin(env: Env) -> Result<(), Error> {
        require_cooldown(&env)?;
        let pending = storage::get_pending_admin(&env).ok_or(Error::NoPendingAdmin)?;
        pending.require_auth();
        let old_admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        storage::set_admin(&env, &pending);
        storage::clear_pending_admin(&env);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::admin_transfer_completed(&env, &old_admin, &pending);
        Ok(())
    }

    pub fn get_pending_admin(env: Env) -> Option<Address> {
        storage::get_pending_admin(&env)
    }

    pub fn get_limits(_env: Env) -> ConfiguredLimits {
        ConfiguredLimits {
            max_amount: MAX_AMOUNT,
            max_expiry_window: MAX_EXPIRY_WINDOW,
            max_total_escrowed: MAX_TOTAL_ESCROWED,
            max_page_size: MAX_PAGE_SIZE,
        }
    }
}
