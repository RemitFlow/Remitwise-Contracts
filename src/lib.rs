#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

mod error;
mod events;
mod storage;
mod types;

use soroban_sdk::contract;

/// The RemitFlow remittance escrow contract.
#[contract]
pub struct RemitFlowContract;
