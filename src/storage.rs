use soroban_sdk::{contracttype, Address, BytesN, Env, IntoVal, Val};

use crate::types::Transfer;

/// Number of ledgers used as the threshold before bumping instance TTL.
pub const INSTANCE_BUMP_THRESHOLD: u32 = 518_400;
/// Number of ledgers the instance TTL is extended to when bumped.
pub const INSTANCE_BUMP_AMOUNT: u32 = 535_680;
/// Number of ledgers used as the threshold before bumping persistent TTL.
pub const PERSISTENT_BUMP_THRESHOLD: u32 = 518_400;
/// Number of ledgers the persistent TTL is extended to when bumped.
pub const PERSISTENT_BUMP_AMOUNT: u32 = 535_680;

/// The shorter TTL used by allowlist entries.  Allowlist membership is
/// configuration, not escrow, so it should be refreshed when used without
/// making a dormant address keep state alive indefinitely.
pub const CALLER_BUMP_THRESHOLD: u32 = 259_200;
pub const CALLER_BUMP_AMOUNT: u32 = 276_480;

/// Account quotas are operational metadata and have a smaller retention
/// window than a live transfer.  The entry is still refreshed on every
/// successful operation, so an active account does not lose its quota.
pub const ACCOUNT_OP_BUMP_THRESHOLD: u32 = 86_400;
pub const ACCOUNT_OP_BUMP_AMOUNT: u32 = 95_040;

/// Terminal transfers are retained briefly for status/audit queries.  A
/// caller may explicitly clean them earlier through the bounded cleanup
/// entrypoint below.
pub const TERMINAL_BUMP_THRESHOLD: u32 = 10_080;
pub const TERMINAL_BUMP_AMOUNT: u32 = 20_160;

/// Maximum number of terminal ids processed by one cleanup invocation.
pub const MAX_TERMINAL_CLEANUP: u32 = 20;

/// Keys for values held in **instance** storage.
///
/// Instance storage shares its time-to-live with the contract instance itself
/// and is extended on every mutating call via [extend_instance]. All
/// singleton configuration values live here.
///
/// # Collision safety
/// Soroban serialises #[contracttype] enum keys as an XDR ScVec whose
/// first element is the variant name as a Symbol. Because the name string is
/// part of the on-chain key, no two distinct variants - even with identical
/// payloads - can ever collide. Separating instance and persistent keys into
/// two enums makes a mis-routed write (e.g. passing an [InstanceKey] to the
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
    /// Cumulative amount accepted into escrow across the contract lifetime.
    TotalFunded,
    /// Cumulative amount released from escrow by claims and refunds.
    TotalReleased,
    /// Ledger timestamp at which initialize was called.
    InitializedAt,
    /// Timestamp of the most recent privileged administrative call.
    LastPrivilegedCall,
    /// Hash of the wasm artifact currently recorded as active.
    UpgradeArtifactHash,
    /// Monotonic release number for the active wasm artifact.
    UpgradeVersion,
    /// Monotonic version of the caller registry.
    CallerRegistryVersion,
}

/// Keys for values held in **persistent** storage.
///
/// Persistent entries have their own TTL, extended individually when written.
/// Per-transfer records and the caller allowlist live here because they grow
/// unboundedly and must outlive the instance entry TTL.
///
/// # Collision safety
/// Transfer(u64) and AllowedCaller(Address) can never collide: their
/// serialised keys differ by variant name string ("Transfer" vs
/// "AllowedCaller"), regardless of the payload value. See [InstanceKey]
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
    /// Idempotency receipt for a completed batch invocation.
    Batch(u64),
    /// Replay marker for an exact versioned caller update.
    CallerUpdate(u64, Address, bool),
}

/// Storage retention policy for each persistent record class.
///
/// Keeping the policy in one place makes it possible to audit every write and
/// prevents a new caller from accidentally using the long-lived transfer
/// policy for a short-lived operational record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersistentTtlClass {
    ActiveTransfer,
    TerminalTransfer,
    CallerAllowlist,
    AccountOperations,
}

impl PersistentTtlClass {
    fn limits(self) -> (u32, u32) {
        match self {
            Self::ActiveTransfer => (PERSISTENT_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT),
            Self::TerminalTransfer => (TERMINAL_BUMP_THRESHOLD, TERMINAL_BUMP_AMOUNT),
            Self::CallerAllowlist => (CALLER_BUMP_THRESHOLD, CALLER_BUMP_AMOUNT),
            Self::AccountOperations => (ACCOUNT_OP_BUMP_THRESHOLD, ACCOUNT_OP_BUMP_AMOUNT),
        }
    }
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

/// Apply a persistent TTL policy after a write.
///
/// Soroban's `extend_ttl` is monotonic: it only raises an entry to the
/// requested horizon when it is below the threshold.  Calling this helper on
/// every state transition is therefore idempotent and does not repeatedly
/// rewrite an entry that is already healthy.
fn extend_persistent<K: IntoVal<Env, Val>>(env: &Env, key: &K, class: PersistentTtlClass) {
    let (threshold, amount) = class.limits();
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, amount);
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

/// Store the ledger timestamp at which the contract was initialized.
pub fn set_initialized_at(env: &Env, timestamp: u64) {
    env.storage()
        .instance()
        .set(&InstanceKey::InitializedAt, &timestamp);
}

/// Read the ledger timestamp at which the contract was initialized, if any.
pub fn get_initialized_at(env: &Env) -> Option<u64> {
    env.storage().instance().get(&InstanceKey::InitializedAt)
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

/// Read the cumulative amount that has entered escrow.
pub fn get_total_funded(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&InstanceKey::TotalFunded)
        .unwrap_or(0)
}

/// Persist the cumulative amount that has entered escrow.
pub fn set_total_funded(env: &Env, value: i128) {
    env.storage()
        .instance()
        .set(&InstanceKey::TotalFunded, &value);
}

/// Read the cumulative amount released through claims or refunds.
pub fn get_total_released(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&InstanceKey::TotalReleased)
        .unwrap_or(0)
}

/// Persist the cumulative amount released through claims or refunds.
pub fn set_total_released(env: &Env, value: i128) {
    env.storage()
        .instance()
        .set(&InstanceKey::TotalReleased, &value);
}

/// Read the timestamp of the last privileged call (0 when unset).
pub fn get_last_privileged_call(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&InstanceKey::LastPrivilegedCall)
        .unwrap_or(0)
}

pub fn set_upgrade_artifact_hash(env: &Env, hash: &BytesN<32>) {
    env.storage()
        .instance()
        .set(&InstanceKey::UpgradeArtifactHash, hash);
}

pub fn get_upgrade_artifact_hash(env: &Env) -> Option<BytesN<32>> {
    env.storage()
        .instance()
        .get(&InstanceKey::UpgradeArtifactHash)
}

pub fn set_upgrade_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&InstanceKey::UpgradeVersion, &version);
}

pub fn get_upgrade_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&InstanceKey::UpgradeVersion)
        .unwrap_or(0)
}

/// Persist the timestamp of the last privileged call.
pub fn set_last_privileged_call(env: &Env, timestamp: u64) {
    env.storage()
        .instance()
        .set(&InstanceKey::LastPrivilegedCall, &timestamp);
}

/// Read the current caller registry version, defaulting to zero.
pub fn get_caller_registry_version(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&InstanceKey::CallerRegistryVersion)
        .unwrap_or(0)
}

/// Persist the current caller registry version.
pub fn set_caller_registry_version(env: &Env, version: u64) {
    env.storage()
        .instance()
        .set(&InstanceKey::CallerRegistryVersion, &version);
}

// ---------------------------------------------------------------------------
// Persistent storage helpers
// ---------------------------------------------------------------------------

/// Store a transfer record in persistent storage keyed by its id.
pub fn set_transfer(env: &Env, transfer: &Transfer) {
    let key = PersistentKey::Transfer(transfer.id);
    env.storage().persistent().set(&key, transfer);
    let class = if transfer.status == crate::types::Status::Pending {
        PersistentTtlClass::ActiveTransfer
    } else {
        PersistentTtlClass::TerminalTransfer
    };
    extend_persistent(env, &key, class);
}

/// Read a transfer record from persistent storage by id, if present.
pub fn get_transfer(env: &Env, id: u64) -> Option<Transfer> {
    env.storage().persistent().get(&PersistentKey::Transfer(id))
}

/// Returns true if a transfer with the given id exists.
pub fn has_transfer(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&PersistentKey::Transfer(id))
}

/// Store the receipt for a completed idempotent batch invocation.
pub fn set_batch_receipt(env: &Env, batch_id: u64, receipt: &crate::types::BatchReceipt) {
    let key = PersistentKey::Batch(batch_id);
    env.storage().persistent().set(&key, receipt);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

/// Read an idempotent batch receipt, if the batch id has been completed.
pub fn get_batch_receipt(env: &Env, batch_id: u64) -> Option<crate::types::BatchReceipt> {
    env.storage()
        .persistent()
        .get(&PersistentKey::Batch(batch_id))
}

/// Store a caller's allowlist status in persistent storage.
pub fn set_caller_allowed(env: &Env, caller: &Address, allowed: bool) {
    let key = PersistentKey::AllowedCaller(caller.clone());
    if allowed {
        env.storage().persistent().set(&key, &true);
        extend_persistent(env, &key, PersistentTtlClass::CallerAllowlist);
    } else {
        env.storage().persistent().remove(&key);
    }
}

/// Check if a caller is allowed from persistent storage.
pub fn get_account_op_count(env: &Env, account: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&PersistentKey::AccountOpCount(account.clone()))
        .unwrap_or(0)
}

pub fn increment_account_op_count(env: &Env, account: &Address) {
    let count: u32 = get_account_op_count(env, account);
    let key = PersistentKey::AccountOpCount(account.clone());
    env.storage()
        .persistent()
        .set(&key, &(count.saturating_add(1)));
    extend_persistent(env, &key, PersistentTtlClass::AccountOperations);
}

pub fn is_caller_allowed(env: &Env, caller: &Address) -> bool {
    let key = PersistentKey::AllowedCaller(caller.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

/// Remove a transfer only after it has reached a terminal state.
///
/// Returning a boolean keeps cleanup idempotent: already-removed ids and
/// unknown ids cost one bounded lookup and do not turn a maintenance sweep
/// into a failing transaction.  Pending records are deliberately untouched.
pub fn remove_terminal_transfer(env: &Env, id: u64) -> bool {
    let key = PersistentKey::Transfer(id);
    match env.storage().persistent().get::<_, Transfer>(&key) {
        Some(transfer) if transfer.status != crate::types::Status::Pending => {
            env.storage().persistent().remove(&key);
            true
        },
        _ => false,
    }
}

/// Check whether an exact versioned caller update has already been applied.
pub fn has_caller_update(env: &Env, version: u64, caller: &Address, allowed: bool) -> bool {
    env.storage()
        .persistent()
        .get(&PersistentKey::CallerUpdate(version, caller.clone(), allowed))
        .unwrap_or(false)
}

/// Mark an exact versioned caller update as applied.
pub fn set_caller_update(env: &Env, version: u64, caller: &Address, allowed: bool) {
    let key = PersistentKey::CallerUpdate(version, caller.clone(), allowed);
    env.storage().persistent().set(&key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
