use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Publish an event recording contract initialization.
pub fn init(env: &Env, admin: &Address, token: &Address) {
    let topics = (Symbol::new(env, "init"),);
    env.events().publish(topics, (admin.clone(), token.clone()));
}

/// Publish an event recording the creation of a new transfer.
pub fn created(env: &Env, id: u64, from: &Address, recipient: &Address, amount: i128, expiry: u64) {
    let topics = (Symbol::new(env, "created"), id);
    env.events()
        .publish(topics, (from.clone(), recipient.clone(), amount, expiry));
}

/// Publish an event recording a successful claim by the recipient.
pub fn claimed(env: &Env, id: u64, recipient: &Address, amount: i128) {
    let topics = (Symbol::new(env, "claimed"), id);
    env.events().publish(topics, (recipient.clone(), amount));
}

/// Publish an event recording a cancellation and refund to the sender.
pub fn cancelled(env: &Env, id: u64, from: &Address, amount: i128) {
    let topics = (Symbol::new(env, "cancelled"), id);
    env.events().publish(topics, (from.clone(), amount));
}

/// Publish an event recording that the admin paused the contract.
pub fn paused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "paused"),);
    env.events().publish(topics, admin.clone());
}

/// Publish an event recording that the admin unpaused the contract.
pub fn unpaused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "unpaused"),);
    env.events().publish(topics, admin.clone());
}

/// Publish an event recording that a caller was added to the allowlist.
pub fn caller_added(env: &Env, caller: &Address) {
    let topics = (Symbol::new(env, "caller_added"),);
    env.events().publish(topics, caller.clone());
}

/// Publish an event recording that a caller was removed from the allowlist.
pub fn caller_removed(env: &Env, caller: &Address) {
    let topics = (Symbol::new(env, "caller_removed"),);
    env.events().publish(topics, caller.clone());
}

/// Publish an event recording that the current admin has nominated a new admin.
///
/// Emitted by `transfer_admin`. The transfer is not yet complete; the nominee
/// must call `accept_admin` to finalise it.
pub fn admin_transfer_started(env: &Env, current_admin: &Address, pending_admin: &Address) {
    let topics = (Symbol::new(env, "admin_transfer_started"),);
    env.events()
        .publish(topics, (current_admin.clone(), pending_admin.clone()));
}

/// Publish an event recording that the pending admin has accepted ownership.
///
/// Emitted by `accept_admin`. `old_admin` is the previous administrator and
/// `new_admin` is the address that now holds the role.
pub fn admin_transfer_completed(env: &Env, old_admin: &Address, new_admin: &Address) {
    let topics = (Symbol::new(env, "admin_transfer_completed"),);
    env.events()
        .publish(topics, (old_admin.clone(), new_admin.clone()));
}

// ---------------------------------------------------------------------------
// Savings goal events
//
// Unlike the tuple payloads above, these use explicit #[contracttype]
// payload structs so every field is named and the schema is self-describing
// for indexers consuming goal lifecycle events (see docs/event-reference.md).
// ---------------------------------------------------------------------------

/// Payload for the `goal_created` event.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalCreatedEvent {
    pub goal_id: u64,
    pub owner: Address,
    pub target_amount: i128,
    pub deadline: u64,
    pub timestamp: u64,
}

/// Payload for the `goal_deposited` event. `amount` is the deposit delta;
/// `new_total` is the goal's `current_amount` after applying it.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalDepositedEvent {
    pub goal_id: u64,
    pub owner: Address,
    pub amount: i128,
    pub new_total: i128,
    pub timestamp: u64,
}

/// Payload for the `goal_withdrawn` event. `amount` is the withdrawal delta;
/// `new_total` is the goal's `current_amount` after applying it.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalWithdrawnEvent {
    pub goal_id: u64,
    pub owner: Address,
    pub amount: i128,
    pub new_total: i128,
    pub timestamp: u64,
}

/// Payload for the `goal_completed` event, emitted when a deposit brings
/// `current_amount` to or past `target_amount`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalCompletedEvent {
    pub goal_id: u64,
    pub owner: Address,
    pub final_amount: i128,
    pub timestamp: u64,
}

/// Payload for the `goal_cancelled` event.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalCancelledEvent {
    pub goal_id: u64,
    pub owner: Address,
    pub refunded_amount: i128,
    pub timestamp: u64,
}

/// Publish an event recording the creation of a new savings goal.
pub fn goal_created(env: &Env, payload: &GoalCreatedEvent) {
    let topics = (Symbol::new(env, "goal_created"), payload.goal_id);
    env.events().publish(topics, payload.clone());
}

/// Publish an event recording a deposit toward a savings goal.
pub fn goal_deposited(env: &Env, payload: &GoalDepositedEvent) {
    let topics = (Symbol::new(env, "goal_deposited"), payload.goal_id);
    env.events().publish(topics, payload.clone());
}

/// Publish an event recording a withdrawal from a savings goal.
pub fn goal_withdrawn(env: &Env, payload: &GoalWithdrawnEvent) {
    let topics = (Symbol::new(env, "goal_withdrawn"), payload.goal_id);
    env.events().publish(topics, payload.clone());
}

/// Publish an event recording that a savings goal reached its target.
pub fn goal_completed(env: &Env, payload: &GoalCompletedEvent) {
    let topics = (Symbol::new(env, "goal_completed"), payload.goal_id);
    env.events().publish(topics, payload.clone());
}

/// Publish an event recording that a savings goal was cancelled and any
/// balance refunded to the owner.
pub fn goal_cancelled(env: &Env, payload: &GoalCancelledEvent) {
    let topics = (Symbol::new(env, "goal_cancelled"), payload.goal_id);
    env.events().publish(topics, payload.clone());
}
