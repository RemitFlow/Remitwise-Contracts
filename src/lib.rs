#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

mod error;
mod events;
pub mod math;
mod storage;
mod types;

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_utils;

use soroban_sdk::{contract, contractimpl, contractmeta, token, Address, Env, Vec};

use crate::error::Error;
use crate::types::{Status, Transfer};

contractmeta!(key = "name", val = "RemitFlow");
contractmeta!(key = "version", val = "0.1.0");
contractmeta!(
    key = "description",
    val = "Cross-border remittance escrow for Soroban/Stellar"
);

/// Largest token amount accepted for a single escrowed transfer.
///
/// Bounds individual transfers to guard against accidental or malicious
/// outsized values while staying well within the token's `i128` range.
pub const MAX_AMOUNT: i128 = 1_000_000_000_000_000_000;

/// Maximum allowed distance, in seconds, between now and a transfer's expiry.
///
/// Caps how far in the future an escrow can be scheduled (roughly one year)
/// so funds are not locked away indefinitely by an out-of-range expiry.
pub const MAX_EXPIRY_WINDOW: u64 = 31_536_000;

/// Global cap on the total escrowed amount.
///
/// Prevents the contract from accumulating an unbounded escrow balance.
pub const MAX_TOTAL_ESCROWED: i128 = MAX_AMOUNT;

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

    /// Return the configured administrator address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Return the configured token contract address.
    pub fn get_token(env: Env) -> Result<Address, Error> {
        storage::get_token(&env).ok_or(Error::NotInitialized)
    }

    /// Return the number of transfers created so far.
    pub fn counter(env: Env) -> u64 {
        storage::get_counter(&env)
    }

    /// Pause the contract, blocking creation of new transfers.
    ///
    /// Only the administrator may pause. Claims and cancellations of
    /// existing transfers remain available while paused.
    pub fn pause(env: Env) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_paused(&env, true);
        storage::extend_instance(&env);
        events::paused(&env, &admin);
        Ok(())
    }

    /// Unpause the contract, re-enabling creation of new transfers.
    ///
    /// Only the administrator may unpause.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_paused(&env, false);
        storage::extend_instance(&env);
        events::unpaused(&env, &admin);
        Ok(())
    }

    /// Create a new escrowed transfer from `from` to `recipient`.
    ///
    /// Transfers `amount` of the configured token from `from` into the
    /// contract and records a pending transfer that expires at `expiry`.
    /// Returns the new transfer's id.
    pub fn create_transfer(
        env: Env,
        from: Address,
        recipient: Address,
        amount: i128,
        expiry: u64,
    ) -> Result<u64, Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        if storage::get_paused(&env) {
            return Err(Error::ContractPaused);
        }
        if !storage::is_caller_allowed(&env, &from) {
            return Err(Error::CallerNotAllowed);
        }
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if amount > MAX_AMOUNT {
            return Err(Error::AmountTooLarge);
        }
        let total_escrowed = Self::total_escrowed(env.clone());
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
        storage::extend_instance(&env);
        events::created(&env, id, &from, &recipient, amount, expiry);
        Ok(id)
    }

    /// Claim a pending transfer, releasing its funds to the recipient.
    ///
    /// Only the recorded recipient may claim, the transfer must still be
    /// pending, and the current ledger time must not exceed the expiry.
    pub fn claim_transfer(env: Env, id: u64, recipient: Address) -> Result<(), Error> {
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
        let amount = transfer.amount;
        storage::set_transfer(&env, &transfer);
        storage::extend_instance(&env);
        events::claimed(&env, id, &recipient, amount);
        Ok(())
    }

    /// Cancel a pending transfer after expiry, refunding the sender.
    ///
    /// Only the original sender may cancel, the transfer must still be
    /// pending, and the expiry must have passed.
    pub fn cancel_transfer(env: Env, id: u64, from: Address) -> Result<(), Error> {
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
        let amount = transfer.amount;
        storage::set_transfer(&env, &transfer);
        storage::extend_instance(&env);
        events::cancelled(&env, id, &from, amount);
        Ok(())
    }

    /// Return true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        storage::get_paused(&env)
    }

    /// Fetch the full transfer record for the given id.
    pub fn get_transfer(env: Env, id: u64) -> Result<Transfer, Error> {
        storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)
    }

    /// Return true if a transfer with the given id has been recorded.
    pub fn transfer_exists(env: Env, id: u64) -> bool {
        storage::has_transfer(&env, id)
    }

    /// Return just the lifecycle status of the transfer with the given id.
    pub fn get_status(env: Env, id: u64) -> Result<Status, Error> {
        storage::get_transfer(&env, id)
            .map(|transfer| transfer.status)
            .ok_or(Error::TransferNotFound)
    }

    /// Return a page of transfer records starting at `start_id`.
    ///
    /// Collects up to `limit` existing transfers with ids in
    /// `start_id..=counter`, skipping any gaps. A `limit` of zero yields an
    /// empty page.
    pub fn get_transfers_paged(env: Env, start_id: u64, limit: u32) -> Vec<Transfer> {
        let last = storage::get_counter(&env);
        let mut page = Vec::new(&env);
        let mut id = start_id.max(1);
        while id <= last && page.len() < limit {
            if let Some(transfer) = storage::get_transfer(&env, id) {
                page.push_back(transfer);
            }
            id += 1;
        }
        page
    }

    /// Return the total token amount currently held in escrow.
    ///
    /// Sums the amounts of every transfer still in [`Status::Pending`] across
    /// ids `1..=counter`. Uses saturating addition so the total never wraps.
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

    /// Return true if the transfer with the given id has passed its expiry.
    ///
    /// Compares the transfer's `expiry` against the current ledger
    /// timestamp; the lifecycle status is not considered.
    pub fn is_expired(env: Env, id: u64) -> Result<bool, Error> {
        let transfer = storage::get_transfer(&env, id).ok_or(Error::TransferNotFound)?;
        Ok(env.ledger().timestamp() > transfer.expiry)
    }

    /// Count how many created transfers were funded by `from`.
    ///
    /// Scans transfer ids `1..=counter` and tallies records whose sender
    /// matches `from`.
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

    /// Count how many created transfers target `recipient`.
    ///
    /// Scans transfer ids `1..=counter` and tallies records whose recipient
    /// matches `recipient`.
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

    /// Count how many created transfers currently hold the given status.
    ///
    /// Scans transfer ids `1..=counter` and tallies records whose
    /// [`Status`] matches `status`.
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

    /// Add a caller to the allowlist of privileged callers.
    ///
    /// Only the administrator may add callers.
    pub fn add_caller(env: Env, caller: Address) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_caller_allowed(&env, &caller, true);
        storage::extend_instance(&env);
        events::caller_added(&env, &caller);
        Ok(())
    }

    /// Remove a caller from the allowlist of privileged callers.
    ///
    /// Only the administrator may remove callers.
    pub fn remove_caller(env: Env, caller: Address) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();
        storage::set_caller_allowed(&env, &caller, false);
        storage::extend_instance(&env);
        events::caller_removed(&env, &caller);
        Ok(())
    }

    /// Return true if the caller is on the privileged callers allowlist.
    pub fn is_caller_allowed(env: Env, caller: Address) -> bool {
        storage::is_caller_allowed(&env, &caller)
    }
}
