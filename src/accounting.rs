//! Single-owner accounting for every escrow balance mutation.
//!
//! The token contract is the source of custody, but this contract must also
//! maintain a durable liability ledger. Keeping all liability transitions in
//! this module makes it difficult for a new lifecycle entrypoint to update a
//! transfer record without updating the aggregate totals that protect it.

use soroban_sdk::{token, Address, Env};

use crate::{math, storage, Error, MAX_TOTAL_ESCROWED};

/// Internal accounting façade for escrow funding and release operations.
pub struct EscrowAccounting;

impl EscrowAccounting {
    /// Check whether a new escrow can be recorded without exceeding a limit or
    /// overflowing the lifetime funded counter.
    pub fn validate_funding(env: &Env, amount: i128) -> Result<(), Error> {
        let pending = storage::get_total_escrowed(env);
        let next_pending =
            math::checked_add_amount(pending, amount).ok_or(Error::AccountingOverflow)?;
        if next_pending > MAX_TOTAL_ESCROWED {
            return Err(Error::EscrowCapReached);
        }

        math::checked_add_amount(storage::get_total_funded(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        Ok(())
    }

    /// Check whether a terminal transition can release an escrow amount.
    pub fn validate_release(env: &Env, amount: i128) -> Result<(), Error> {
        let pending = storage::get_total_escrowed(env);
        if amount < 0 || pending < amount {
            return Err(Error::AccountingOverflow);
        }
        math::checked_sub_amount(pending, amount).ok_or(Error::AccountingOverflow)?;
        math::checked_add_amount(storage::get_total_released(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        Ok(())
    }

    /// Record a successful token transfer into escrow.
    pub fn record_funding(env: &Env, amount: i128) -> Result<(), Error> {
        Self::validate_funding(env, amount)?;

        let next_pending = math::checked_add_amount(storage::get_total_escrowed(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        let next_funded = math::checked_add_amount(storage::get_total_funded(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        storage::set_total_escrowed(env, next_pending);
        storage::set_total_funded(env, next_funded);
        Ok(())
    }

    /// Record a successful token transfer out of escrow.
    pub fn record_release(env: &Env, amount: i128) -> Result<(), Error> {
        Self::validate_release(env, amount)?;

        let next_pending = math::checked_sub_amount(storage::get_total_escrowed(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        let next_released = math::checked_add_amount(storage::get_total_released(env), amount)
            .ok_or(Error::AccountingOverflow)?;
        storage::set_total_escrowed(env, next_pending);
        storage::set_total_released(env, next_released);
        Ok(())
    }

    /// Verify both internal conservation and external token solvency.
    pub fn assert_invariant(env: &Env, token: &Address) -> Result<(), Error> {
        let expected_funded = math::checked_add_amount(
            storage::get_total_escrowed(env),
            storage::get_total_released(env),
        )
        .ok_or(Error::AccountingOverflow)?;

        if storage::get_total_funded(env) != expected_funded {
            return Err(Error::SupplyInvariantViolation);
        }

        let balance = token::Client::new(env, token).balance(&env.current_contract_address());
        if balance < storage::get_total_escrowed(env) {
            return Err(Error::SupplyInvariantViolation);
        }
        Ok(())
    }
}
