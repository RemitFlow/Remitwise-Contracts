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

/// Keys for values held in **instance** storage.
///
/// Instance storage shares its time-to-live with the contract instance itself
/// and is extended on every mutating call via [`extend_instance`]. All
/// singleton configuration values live here.
///
/// # Collision safety
/// Soroban serialises `#[contracttype]` enum keys as an XDR `ScVec` whose
/// first element is the variant name as a `Symbol`. Because the name string is
/// part of the on-chain key, no two distinct variants — even with identical
/// payloads — can ever collide. Separating instance and persistent keys into
/// two enums makes a mis-routed write (e.g. passing an [`InstanceKey`] to the
/// persistent store) a compile error rather than a silent bug.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstanceKey {
    /// Administrator address.
    Admin,
    /// Nominated successor awaiting acceptance (instance storage).
    ///
    /// Present only while a two-step admin transfer is in progress.
    PendingAdmin,
    /// Token contract address used for escrow transfers.
    Token,
    /// Monotonic counter for issued transfer ids.
    Counter,
    /// Paused flag gating new transfer creation.
    Paused,
    /// Running total of all currently pending escrowed amounts.
    ///
    /// Maintained incrementally on create/claim/cancel so that creating a
    /// transfer stays O(1) instead of rescanning every stored transfer.
    TotalEscrowed,
}

/// Keys for values held in **persistent** storage.
///
/// Persistent entries have their own TTL, extended individually when written.
/// Per-transfer records and the caller allowlist live here because they grow
/// unboundedly and must outlive the instance entry TTL.
///
/// # Collision safety
/// `Transfer(u64)` and `AllowedCaller(Address)` can never collide: their
/// serialised keys differ by variant name string (`"Transfer"` vs
/// `"AllowedCaller"`), regardless of the payload value. See [`InstanceKey`]
/// for the full encoding note.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistentKey {
    /// A single transfer record, keyed by its unique sequential id.
    Transfer(u64),
    /// Allowlist membership flag for a privileged caller address.
    AllowedCaller(Address),
    /// Per-account operation counter, keyed by account address.
    AccountOpCount(Address),
}

// ---------------------------------------------------------------------------
// Instance storage helpers
// ---------------------------------------------------------------------------

/// Extend the time-to-live of the contract instance storage entry.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Store the administrator address in instance storage.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&InstanceKey::Admin, admin);
}

/// Read the administrator address from instance storage.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&InstanceKey::Admin)
}

/// Returns true if the administrator has already been configured.
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&InstanceKey::Admin)
}

/// Store the pending (nominee) admin address in instance storage.
pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage()
        .instance()
        .set(&InstanceKey::PendingAdmin, pending);
}

/// Read the pending (nominee) admin address from instance storage, if any.
pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&InstanceKey::PendingAdmin)
}

/// Remove the pending admin entry from instance storage.
pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&InstanceKey::PendingAdmin);
}

/// Store the token contract address in instance storage.
pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&InstanceKey::Token, token);
}

/// Read the token contract address from instance storage.
pub fn get_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&InstanceKey::Token)
}

/// Read the current transfer counter, defaulting to zero when unset.
pub fn get_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&InstanceKey::Counter)
        .unwrap_or(0)
}

/// Persist a new value for the transfer counter.
pub fn set_counter(env: &Env, value: u64) {
    env.storage().instance().set(&InstanceKey::Counter, &value);
}

/// Read the paused flag, defaulting to false when unset.
pub fn get_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&InstanceKey::Paused)
        .unwrap_or(false)
}

/// Persist the paused flag value.
pub fn set_paused(env: &Env, value: bool) {
    env.storage().instance().set(&InstanceKey::Paused, &value);
}

/// Read the running total of pending escrowed amounts (0 when unset).
pub fn get_total_escrowed(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&InstanceKey::TotalEscrowed)
        .unwrap_or(0)
}

/// Persist the running total of pending escrowed amounts.
pub fn set_total_escrowed(env: &Env, value: i128) {
    env.storage()
        .instance()
        .set(&InstanceKey::TotalEscrowed, &value);
}

// ---------------------------------------------------------------------------
// Persistent storage helpers
// ---------------------------------------------------------------------------

/// Store a transfer record in persistent storage keyed by its id.
pub fn set_transfer(env: &Env, transfer: &Transfer) {
    let key = PersistentKey::Transfer(transfer.id);
    env.storage().persistent().set(&key, transfer);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Read a transfer record from persistent storage by id, if present.
pub fn get_transfer(env: &Env, id: u64) -> Option<Transfer> {
    env.storage().persistent().get(&PersistentKey::Transfer(id))
}

/// Returns true if a transfer with the given id exists.
pub fn has_transfer(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&PersistentKey::Transfer(id))
}

/// Store a caller's allowlist status in persistent storage.
pub fn set_caller_allowed(env: &Env, caller: &Address, allowed: bool) {
    let key = PersistentKey::AllowedCaller(caller.clone());
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
pub fn get_account_op_count(env: &Env, account: &Address) -> u32 {
    env.storage()
        .instance()
        .get(&PersistentKey::AccountOpCount(account.clone()))
        .unwrap_or(0)
}

pub fn increment_account_op_count(env: &Env, account: &Address) {
    let count: u32 = get_account_op_count(env, account);
    env.storage().instance().set(
        &PersistentKey::AccountOpCount(account.clone()),
        &(count.saturating_add(1)),
    );
}

pub fn is_caller_allowed(env: &Env, caller: &Address) -> bool {
    let key = PersistentKey::AllowedCaller(caller.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}
