#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

// soroban #[contracttype] generates Arbitrary impls under 	estutils,
// which need std. Link it for test builds only; wasm builds stay no_std.
#[cfg(test)]
extern crate std;

mod accounting;
mod error;
mod events;
pub mod math;
mod storage;
mod types;

#[cfg(test)]
mod batch_tests;
mod accounting_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod test;
#[cfg(test)]
mod test_upgrade;
mod event_schema_tests;
mod test_utils;

use soroban_sdk::{contract, contractimpl, contractmeta, token, Address, BytesN, Env, Vec};

use crate::accounting::EscrowAccounting;
use crate::error::Error;
use crate::types::{
    BatchOperation, BatchOperationResult, BatchReceipt, ConfiguredLimits, Status, Transfer,
};
use crate::types::CallerUpdateResult;

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
/// Maximum number of operations allowed in a single batch_operations call.
pub const MAX_BATCH_SIZE: u32 = 50;
/// Maximum number of transfer ids inspected by one expiry sweep batch.
///
/// The limit bounds both storage reads and token transfers, keeping a sweep
/// invocation within a predictable transaction budget.
pub const MAX_SWEEP_BATCH_SIZE: u32 = 50;

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

/// Execute a batch atomically and preserve one result slot per input item.
fn execute_batch_operations(
    env: Env,
    operations: Vec<BatchOperation>,
) -> Result<Vec<BatchOperationResult>, Error> {
    if operations.len() > MAX_BATCH_SIZE {
        return Err(Error::BatchTooLarge);
    }

    let mut results = Vec::new(&env);
    for operation in operations.iter() {
        let result = match operation {
            BatchOperation::Create(params) => {
                let id = RemitFlowContract::create_transfer(
                    env.clone(),
                    params.from,
                    params.recipient,
                    params.amount,
                    params.expiry,
                )?;
                BatchOperationResult::Created(id)
            },
            BatchOperation::Claim(params) => {
                RemitFlowContract::claim_transfer(env.clone(), params.id, params.recipient)?;
                BatchOperationResult::Claimed
            },
            BatchOperation::Cancel(params) => {
                RemitFlowContract::cancel_transfer(env.clone(), params.id, params.from)?;
                BatchOperationResult::Cancelled
            },
        };
        results.push_back(result);
    }
    Ok(results)
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
        execute_batch_operations(env, operations)
    }

    /// Execute an atomic batch with a durable idempotency key.
    ///
    /// A successful retry with the same `batch_id` and identical operations
    /// returns the original indexed result vector without reapplying any
    /// transfer. Reusing an id with a different payload fails closed.
    pub fn batch_operations_idempotent(
        env: Env,
        batch_id: u64,
        operations: Vec<BatchOperation>,
    ) -> Result<Vec<BatchOperationResult>, Error> {
        if batch_id == 0 {
            return Err(Error::InvalidBatchId);
        }

        if let Some(receipt) = storage::get_batch_receipt(&env, batch_id) {
            if receipt.operations != operations {
                return Err(Error::BatchIdConflict);
            }
            return Ok(receipt.results);
        }

        let results = execute_batch_operations(env.clone(), operations.clone())?;
        storage::set_batch_receipt(
            &env,
            batch_id,
            &BatchReceipt {
                operations,
                results: results.clone(),
            },
        );
        storage::extend_instance(&env);
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
        storage::set_total_escrowed(&env, 0);
        storage::set_total_funded(&env, 0);
        storage::set_total_released(&env, 0);
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

    /// Record the currently deployed artifact before enabling replacements.
    /// This is intentionally a one-time, admin-authorized operation.
    pub fn set_upgrade_baseline(env: Env, artifact: BytesN<32>) -> Result<(), Error> {
        if storage::get_upgrade_artifact_hash(&env).is_some() {
            return Err(Error::UpgradeBaselineAlreadySet);
        }
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_upgrade_artifact_hash(&env, &artifact);
        storage::set_upgrade_version(&env, 0);
        storage::extend_instance(&env);
        events::upgrade_baseline_set(&env, &admin, &artifact);
        Ok(())
    }

    /// Replace the current contract wasm only after checking the exact
    /// expected artifact and the next sequential release number.
    pub fn upgrade(
        env: Env,
        expected_artifact: BytesN<32>,
        replacement_artifact: BytesN<32>,
        version: u32,
    ) -> Result<(), Error> {
        require_cooldown(&env)?;
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        let current =
            storage::get_upgrade_artifact_hash(&env).ok_or(Error::UpgradeArtifactMismatch)?;
        if current != expected_artifact {
            return Err(Error::UpgradeArtifactMismatch);
        }
        if current == replacement_artifact {
            return Err(Error::UpgradeArtifactUnchanged);
        }
        let next = storage::get_upgrade_version(&env)
            .checked_add(1)
            .ok_or(Error::UpgradeVersionInvalid)?;
        if version != next {
            return Err(Error::UpgradeVersionInvalid);
        }

        // All validation and audit state are part of this transaction. If the
        // host rejects the artifact, Soroban rolls back both storage and event.
        storage::set_upgrade_artifact_hash(&env, &replacement_artifact);
        storage::set_upgrade_version(&env, version);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::upgrade_applied(&env, &admin, version, &replacement_artifact);
        env.deployer()
            .update_current_contract_wasm(replacement_artifact);
        Ok(())
    }

    pub fn get_upgrade_artifact(env: Env) -> Result<BytesN<32>, Error> {
        storage::get_upgrade_artifact_hash(&env).ok_or(Error::NotInitialized)
    }

    pub fn get_upgrade_version(env: Env) -> u32 {
        storage::get_upgrade_version(&env)
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
        EscrowAccounting::validate_funding(&env, amount)?;
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
        EscrowAccounting::record_funding(&env, amount)?;
        storage::increment_account_op_count(&env, &from);
        EscrowAccounting::assert_invariant(&env, &token)?;
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
        EscrowAccounting::validate_release(&env, transfer.amount)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &recipient,
            &transfer.amount,
        );

        transfer.status = Status::Claimed;
        EscrowAccounting::record_release(&env, transfer.amount)?;
        EscrowAccounting::assert_invariant(&env, &token)?;
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
        EscrowAccounting::validate_release(&env, transfer.amount)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &from,
            &transfer.amount,
        );

        transfer.status = Status::Cancelled;
        EscrowAccounting::record_release(&env, transfer.amount)?;
        EscrowAccounting::assert_invariant(&env, &token)?;
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
        storage::get_total_escrowed(&env)
    }

    /// Return the cumulative amount funded into escrow.
    pub fn total_funded(env: Env) -> i128 {
        storage::get_total_funded(&env)
    }

    /// Return the cumulative amount released by claims and refunds.
    pub fn total_released(env: Env) -> i128 {
        storage::get_total_released(&env)
    }

    pub fn check_supply_invariant(env: Env) -> Result<(), Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        EscrowAccounting::assert_invariant(&env, &token)
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

    /// Apply an admin-authorized caller update at the next registry version.
    ///
    /// Replaying the exact `(version, caller, allowed)` tuple is deterministic
    /// and produces no second event. Different stale updates are rejected.
    pub fn update_caller_versioned(
        env: Env,
        caller: Address,
        allowed: bool,
        version: u64,
    ) -> Result<CallerUpdateResult, Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        require_external_address(&env, &caller)?;
        if storage::has_caller_update(&env, version, &caller, allowed) {
            return Ok(CallerUpdateResult {
                changed: false,
                duplicate: true,
                version: storage::get_caller_registry_version(&env),
            });
        }
        require_cooldown(&env)?;
        let current = storage::get_caller_registry_version(&env);
        let expected = current.checked_add(1).ok_or(Error::CallerUpdateVersionOverflow)?;
        if version != expected {
            return Err(Error::StaleCallerUpdate);
        }

        // The membership bit, version, replay marker, cooldown, and event are
        // one state transition. A trapped call rolls all of them back.
        storage::set_caller_allowed(&env, &caller, allowed);
        storage::set_caller_registry_version(&env, version);
        storage::set_caller_update(&env, version, &caller, allowed);
        record_privileged_call(&env);
        storage::extend_instance(&env);
        events::caller_registry_changed(&env, version, &admin, &caller, allowed);
        Ok(CallerUpdateResult {
            changed: true,
            duplicate: false,
            version,
        })
    }

    /// Return the monotonic version of the allowed-caller registry.
    pub fn caller_registry_version(env: Env) -> u64 {
        storage::get_caller_registry_version(&env)
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

    /// Sweeps an expired transfer, returning the escrowed funds to the original sender.
    ///
    /// Permissionless entrypoint allowing third-party bots to trigger cancellation
    /// and return escrowed funds once expiry has passed.
    pub fn sweep_expired(env: Env, id: u64) -> Result<(), Error> {
        let mut transfer = storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)?;

        if transfer.status != Status::Pending {
            return Err(Error::NotPending);
        }

        if env.ledger().timestamp() <= transfer.expiry {
            return Err(Error::NotExpired);
        }

        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        EscrowAccounting::validate_release(&env, transfer.amount)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &transfer.from,
            &transfer.amount,
        );

        transfer.status = Status::Cancelled;
        EscrowAccounting::record_release(&env, transfer.amount)?;
        EscrowAccounting::assert_invariant(&env, &token)?;

        let amount = transfer.amount;
        let from = transfer.from.clone();

        storage::set_transfer(&env, &transfer);
        storage::extend_instance(&env);
        events::cancelled(&env, id, &from, amount);

        Ok(())
    }

    /// Sweeps expired pending transfers in an ascending, bounded id range.
    ///
    /// `start_id` is inclusive and `0` is clamped to `1`. At most
    /// `min(limit, MAX_SWEEP_BATCH_SIZE)` ids are inspected, including ids
    /// whose records have expired from storage or have already reached a
    /// terminal status. This makes cursored retries deterministic and bounds
    /// the work performed by a single invocation. A transfer is swept only
    /// when the ledger timestamp is strictly greater than its expiry; funds
    /// are returned to its original sender. The call is permissionless and
    /// returns the ids swept during this invocation. Repeating a range is
    /// idempotent because terminal records are skipped.
    pub fn sweep_expired_batch(env: Env, start_id: u64, limit: u32) -> Result<Vec<u64>, Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        let mut swept = Vec::new(&env);
        let mut id = start_id.max(1);
        let mut inspected = 0;
        let max_inspected = limit.min(MAX_SWEEP_BATCH_SIZE);
        let last = storage::get_counter(&env);
        let now = env.ledger().timestamp();

        while id <= last && inspected < max_inspected {
            if let Some(mut transfer) = storage::get_transfer(&env, id) {
                if transfer.status == Status::Pending && now > transfer.expiry {
                    token::Client::new(&env, &token).transfer(
                        &env.current_contract_address(),
                        &transfer.from,
                        &transfer.amount,
                    );
                    transfer.status = Status::Cancelled;
                    storage::set_total_escrowed(
                        &env,
                        storage::get_total_escrowed(&env).saturating_sub(transfer.amount),
                    );
                    let from = transfer.from.clone();
                    let amount = transfer.amount;
                    storage::set_transfer(&env, &transfer);
                    events::cancelled(&env, id, &from, amount);
                    swept.push_back(id);
                }
            }
            inspected += 1;
            match id.checked_add(1) {
                Some(next_id) => id = next_id,
                None => break,
            }
        }

        if !swept.is_empty() {
            EscrowAccounting::assert_invariant(&env, &token)?;
            storage::extend_instance(&env);
        }
        Ok(swept)
    }
}
