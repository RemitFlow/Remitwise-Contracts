#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

use remitflow_contract::{RemitFlowContract, RemitFlowContractClient};
use remitflow_contract::types::Status;

#[derive(Arbitrary, Debug, Clone)]
pub enum Action {
    AdvanceTime {
        delta: u32,
    },
    CreateTransfer {
        amount: i128,
        expiry_delta: u32,
        from_idx: u8,
        recipient_idx: u8,
    },
    ClaimTransfer {
        id_idx: u8,
        recipient_idx: u8,
    },
    CancelTransfer {
        id_idx: u8,
        from_idx: u8,
    },
    SweepExpired {
        id_idx: u8,
    },
}

#[derive(Arbitrary, Debug, Clone)]
pub struct FuzzData {
    pub initial_balances: [i128; 4],
    pub actions: Vec<Action>,
}

fuzz_target!(|data: FuzzData| {
    if data.actions.is_empty() {
        return;
    }

    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    // Create 4 users
    let users = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    let admin = Address::generate(&env);
    
    // Deploy token
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = token_contract.address();
    let token_admin = StellarAssetClient::new(&env, &token_addr);
    let token = TokenClient::new(&env, &token_addr);

    // Give users initial balances (capped to avoid overflow in test setup)
    for (i, &balance) in data.initial_balances.iter().enumerate() {
        let amount = balance.abs() % 1_000_000_000_000_000;
        if amount > 0 {
            token_admin.mint(&users[i], &amount);
        }
    }

    // Deploy contract
    let contract_id = env.register_contract(None, RemitFlowContract);
    let client = RemitFlowContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &token_addr);

    // Allow all users
    for user in &users {
        client.add_caller(&user);
    }

    let mut created_ids = Vec::new();

    // Set initial time
    env.ledger().with_mut(|l| l.timestamp = 10_000_000);

    for action in data.actions {
        // Assert supply invariant before action
        let res_inv = client.try_check_supply_invariant();
        if let Err(Ok(err)) = res_inv {
            panic!("Invariant failed before action: {:?}", err);
        }

        match action {
            Action::AdvanceTime { delta } => {
                let d = (delta % 10_000_000) as u64;
                env.ledger().with_mut(|l| l.timestamp += d);
            }
            Action::CreateTransfer { amount, expiry_delta, from_idx, recipient_idx } => {
                let from = &users[(from_idx % 4) as usize];
                let recipient = &users[(recipient_idx % 4) as usize];
                let expiry = env.ledger().timestamp() + (expiry_delta % 10_000_000) as u64;
                
                let res = client.try_create_transfer(from, recipient, &amount, &expiry);
                if let Ok(Ok(id)) = res {
                    created_ids.push(id);
                }
            }
            Action::ClaimTransfer { id_idx, recipient_idx } => {
                if created_ids.is_empty() {
                    continue;
                }
                let id = created_ids[(id_idx as usize) % created_ids.len()];
                let recipient = &users[(recipient_idx % 4) as usize];
                
                let status_before = client.try_get_status(&id);
                
                let res = client.try_claim_transfer(&id, recipient);
                
                // Monotonic status check
                if let Ok(Ok(status)) = status_before {
                    if status != Status::Pending && res.is_ok() {
                        panic!("Claimed a non-pending transfer");
                    }
                }
            }
            Action::CancelTransfer { id_idx, from_idx } => {
                if created_ids.is_empty() {
                    continue;
                }
                let id = created_ids[(id_idx as usize) % created_ids.len()];
                let from = &users[(from_idx % 4) as usize];
                
                let status_before = client.try_get_status(&id);
                
                let res = client.try_cancel_transfer(&id, from);
                
                // Monotonic status check
                if let Ok(Ok(status)) = status_before {
                    if status != Status::Pending && res.is_ok() {
                        panic!("Cancelled a non-pending transfer");
                    }
                }
            }
            Action::SweepExpired { id_idx } => {
                if created_ids.is_empty() {
                    continue;
                }
                let id = created_ids[(id_idx as usize) % created_ids.len()];
                
                let status_before = client.try_get_status(&id);
                
                let res = client.try_sweep_expired(&id);
                
                if let Ok(Ok(status)) = status_before {
                    if status != Status::Pending && res.is_ok() {
                        panic!("Swept a non-pending transfer");
                    }
                }
            }
        }
        
        // Assert supply invariant after action
        let res_inv = client.try_check_supply_invariant();
        if let Err(Ok(err)) = res_inv {
            panic!("Invariant failed after action: {:?}", err);
        }
    }
});
