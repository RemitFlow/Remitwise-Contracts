use soroban_sdk::{contracttype, Address, Env};

use crate::types::Transfer;

/// Number of ledgers used as the threshold before bumping instance TTL.
pub const INSTANCE_BUMP_THRESHOLD: u32 = 518_400;
/// Number of ledgers the instance TTL is extended to when bumped.
pub const INSTANCE_BUMP_AMOUNT: u32 = 535_680;
/// Number of ledgers used as the threshold before bumping persistent TTL.
pub const PERSISTENT_BUMP_THRESHOLD: u32 = 518_400;
/// Number of ledgers the persistent TTL is extended to when bumped.
pub const PERSISTENT_BUMP_AMOUNT: u32 = 535_680;

/// Keys used to address values in contract storage.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    /// Administrator address (instance storage).
    Admin,
    /// Token contract address used for transfers (instance storage).
    Token,
    /// Monotonic counter for issued transfer ids (instance storage).
    Counter,
    /// A single transfer record keyed by its id (persistent storage).
    Transfer(u64),
}

/// Extend the time-to-live of the instance storage entry.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
