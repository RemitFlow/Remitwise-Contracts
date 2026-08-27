use soroban_sdk::{contracttype, Address, Env, String, Symbol};

/// Current version for every lifecycle event payload.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventMetadata {
    pub schema_version: u32,
    pub amount_unit: String,
    pub timestamp_unit: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitEvent {
    pub metadata: EventMetadata,
    pub admin: Address,
    pub token: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedEvent {
    pub metadata: EventMetadata,
    pub transfer_id: u64,
    pub from: Address,
    pub recipient: Address,
    pub amount: i128,
    pub expiry: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimedEvent {
    pub metadata: EventMetadata,
    pub transfer_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelledEvent {
    pub metadata: EventMetadata,
    pub transfer_id: u64,
    pub from: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorEvent {
    pub metadata: EventMetadata,
    pub actor: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminTransferEvent {
    pub metadata: EventMetadata,
    pub old_admin: Address,
    pub new_admin: Address,
}

fn metadata(env: &Env, amount_unit: &str, timestamp_unit: &str) -> EventMetadata {
    EventMetadata {
        schema_version: EVENT_SCHEMA_VERSION,
        amount_unit: String::from_str(env, amount_unit),
        timestamp_unit: String::from_str(env, timestamp_unit),
    }
}

/// Publish an event recording contract initialization.
pub fn init(env: &Env, admin: &Address, token: &Address) {
    let topics = (Symbol::new(env, "init"),);
    env.events().publish(
        topics,
        InitEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            admin: admin.clone(),
            token: token.clone(),
        },
    );
}

/// Publish an event recording the creation of a new transfer.
pub fn created(env: &Env, id: u64, from: &Address, recipient: &Address, amount: i128, expiry: u64) {
    let topics = (Symbol::new(env, "created"), id);
    env.events().publish(
        topics,
        CreatedEvent {
            metadata: metadata(env, "token_base_units", "ledger_seconds"),
            transfer_id: id,
            from: from.clone(),
            recipient: recipient.clone(),
            amount,
            expiry,
        },
    );
}

/// Publish an event recording a successful claim by the recipient.
pub fn claimed(env: &Env, id: u64, recipient: &Address, amount: i128) {
    let topics = (Symbol::new(env, "claimed"), id);
    env.events().publish(
        topics,
        ClaimedEvent {
            metadata: metadata(env, "token_base_units", ""),
            transfer_id: id,
            recipient: recipient.clone(),
            amount,
        },
    );
}

/// Publish an event recording a cancellation and refund to the sender.
pub fn cancelled(env: &Env, id: u64, from: &Address, amount: i128) {
    let topics = (Symbol::new(env, "cancelled"), id);
    env.events().publish(
        topics,
        CancelledEvent {
            metadata: metadata(env, "token_base_units", ""),
            transfer_id: id,
            from: from.clone(),
            amount,
        },
    );
}

/// Publish an event recording that the admin paused the contract.
pub fn paused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "paused"),);
    env.events().publish(
        topics,
        ActorEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            actor: admin.clone(),
        },
    );
}

/// Publish an event recording that the admin unpaused the contract.
pub fn unpaused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "unpaused"),);
    env.events().publish(
        topics,
        ActorEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            actor: admin.clone(),
        },
    );
}

/// Publish an event recording that a caller was added to the allowlist.
pub fn caller_added(env: &Env, caller: &Address) {
    let topics = (Symbol::new(env, "caller_added"),);
    env.events().publish(
        topics,
        ActorEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            actor: caller.clone(),
        },
    );
}

/// Publish an event recording that a caller was removed from the allowlist.
pub fn caller_removed(env: &Env, caller: &Address) {
    let topics = (Symbol::new(env, "caller_removed"),);
    env.events().publish(
        topics,
        ActorEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            actor: caller.clone(),
        },
    );
}

/// Publish an event recording that the current admin has nominated a new admin.
///
/// Emitted by `transfer_admin`. The transfer is not yet complete; the nominee
/// must call `accept_admin` to finalise it.
pub fn admin_transfer_started(env: &Env, current_admin: &Address, pending_admin: &Address) {
    let topics = (Symbol::new(env, "admin_transfer_started"),);
    env.events().publish(
        topics,
        AdminTransferEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            old_admin: current_admin.clone(),
            new_admin: pending_admin.clone(),
        },
    );
}

/// Publish an event recording that the pending admin has accepted ownership.
///
/// Emitted by `accept_admin`. `old_admin` is the previous administrator and
/// `new_admin` is the address that now holds the role.
pub fn admin_transfer_completed(env: &Env, old_admin: &Address, new_admin: &Address) {
    let topics = (Symbol::new(env, "admin_transfer_completed"),);
    env.events().publish(
        topics,
        AdminTransferEvent {
            metadata: metadata(env, "", "ledger_seconds"),
            old_admin: old_admin.clone(),
            new_admin: new_admin.clone(),
        },
    );
}
