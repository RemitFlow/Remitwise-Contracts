use soroban_sdk::{Address, Env, Symbol};

/// Publish an event recording contract initialization.
pub fn init(env: &Env, admin: &Address, token: &Address) {
    let topics = (Symbol::new(env, "init"),);
    env.events().publish(topics, (admin.clone(), token.clone()));
}

/// Publish an event recording the creation of a new transfer.
pub fn created(env: &Env, id: u64, from: &Address, recipient: &Address, amount: i128) {
    let topics = (Symbol::new(env, "created"), id);
    env.events()
        .publish(topics, (from.clone(), recipient.clone(), amount));
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
