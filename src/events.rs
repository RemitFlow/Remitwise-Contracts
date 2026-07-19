use soroban_sdk::{Address, Env, Symbol};

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

