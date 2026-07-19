use remitflow_contract::{RemitFlowContract, RemitFlowContractClient};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

fn setup() -> (Env, RemitFlowContractClient, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token = token_contract.address();
    let token_admin = StellarAssetClient::new(&env, &token);
    token_admin.mint(&from, &1_000_000_000);

    let contract_id = env.register(RemitFlowContract, ());
    let client = RemitFlowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);

    (env, client, token, from, recipient)
}

fn main() {
    let (env, client, _token, from, recipient) = setup();
    let expiry = env.ledger().timestamp() + 1_000_000;

    // Warm-up
    for _ in 0..5 {
        client.create_transfer(&from, &recipient, &1000, &expiry);
    }

    let iterations = 100;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let amount = 1000 + (i as i128 % 10000);
        client.create_transfer(&from, &recipient, &amount, &expiry);
    }

    let elapsed = start.elapsed();
    let avg = elapsed.as_micros() as f64 / iterations as f64;

    println!("=== create_transfer benchmark ===");
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Avg per call: {:.0} µs", avg);
    println!("Throughput: {:.0} calls/s", 1_000_000.0 / avg);
}
