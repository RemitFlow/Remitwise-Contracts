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
