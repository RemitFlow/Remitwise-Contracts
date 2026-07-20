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
    /// Nominated successor awaiting acceptance (instance storage).
    ///
    /// Present only while a two-step admin transfer is in progress.
    PendingAdmin,
    /// Token contract address used for transfers (instance storage).
    Token,
    /// Monotonic counter for issued transfer ids (instance storage).
    Counter,
    /// Paused flag gating new transfers (instance storage).
    Paused,
    /// A single transfer record keyed by its id (persistent storage).
    Transfer(u64),
    /// Flag indicating whether an address is allowed as a privileged caller (persistent storage).
    AllowedCaller(Address),
}

/// Extend the time-to-live of the instance storage entry.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Store the administrator address in instance storage.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Read the administrator address from instance storage.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

/// Returns true if the administrator has already been configured.
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Store the pending (nominee) admin address in instance storage.
pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::PendingAdmin, pending);
}

/// Read the pending (nominee) admin address from instance storage, if any.
pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}

/// Remove the pending admin entry from instance storage.
pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}

/// Store the token contract address in instance storage.
pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

/// Read the token contract address from instance storage.
pub fn get_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Token)
}

/// Read the current transfer counter, defaulting to zero when unset.
pub fn get_counter(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
}

/// Persist a new value for the transfer counter.
pub fn set_counter(env: &Env, value: u64) {
    env.storage().instance().set(&DataKey::Counter, &value);
}

/// Read the paused flag, defaulting to false when unset.
pub fn get_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

/// Persist the paused flag value.
pub fn set_paused(env: &Env, value: bool) {
    env.storage().instance().set(&DataKey::Paused, &value);
}

/// Store a transfer record in persistent storage keyed by its id.
pub fn set_transfer(env: &Env, transfer: &Transfer) {
    let key = DataKey::Transfer(transfer.id);
    env.storage().persistent().set(&key, transfer);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Read a transfer record from persistent storage by id, if present.
pub fn get_transfer(env: &Env, id: u64) -> Option<Transfer> {
    env.storage().persistent().get(&DataKey::Transfer(id))
}

/// Returns true if a transfer with the given id exists.
pub fn has_transfer(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Transfer(id))
}

/// Store a caller's allowlist status in persistent storage.
pub fn set_caller_allowed(env: &Env, caller: &Address, allowed: bool) {
    let key = DataKey::AllowedCaller(caller.clone());
    if allowed {
        env.storage().persistent().set(&key, &true);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    } else {
        env.storage().persistent().remove(&key);
    }
}

/// Check if a caller is allowed from persistent storage.
pub fn is_caller_allowed(env: &Env, caller: &Address) -> bool {
    let key = DataKey::AllowedCaller(caller.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}
