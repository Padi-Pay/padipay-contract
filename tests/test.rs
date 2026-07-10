#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

#[test]
fn test_create_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000;

    client.create_escrow(&buyer, &seller, &token, &amount);

    env.as_contract(&contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env).unwrap();
        assert_eq!(state.buyer, buyer);
        assert_eq!(state.seller, seller);
        assert_eq!(state.token, token);
        assert_eq!(state.amount, amount);
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_create_escrow_unauthorized() {
    let env = Env::default();
    // Do NOT mock auths here.
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000;

    // This should panic because buyer didn't authorize
    client.create_escrow(&buyer, &seller, &token, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_create_escrow_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 0; // Invalid amount

    client.create_escrow(&buyer, &seller, &token, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_create_escrow_invalid_addresses() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000;

    // Buyer == seller
    client.create_escrow(&buyer, &buyer, &token, &amount);
}

#[test]
fn test_lock_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());
    let token_client_basic = soroban_sdk::token::Client::new(&env, &token_contract.address());

    // Mint tokens to buyer
    token_client.mint(&buyer, &10000);
    assert_eq!(token_client_basic.balance(&buyer), 10000);

    // Create escrow
    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);

    // Lock funds
    client.lock_funds();

    // Check balances
    assert_eq!(token_client_basic.balance(&buyer), 9000);
    assert_eq!(token_client_basic.balance(&contract_id), 1000);

    env.as_contract(&contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_lock_funds_already_funded() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());

    token_client.mint(&buyer, &10000);

    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);
    client.lock_funds();

    // This should panic with AlreadyFunded
    client.lock_funds();
}

#[test]
fn test_release_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());
    let token_client_basic = soroban_sdk::token::Client::new(&env, &token_contract.address());

    // Mint tokens to buyer
    token_client.mint(&buyer, &10000);

    // Create escrow
    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);

    // Lock funds
    client.lock_funds();

    // Release funds
    client.release_funds();

    // Check balances
    assert_eq!(token_client_basic.balance(&contract_id), 0);
    assert_eq!(token_client_basic.balance(&seller), 1000);

    env.as_contract(&contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_release_funds_already_released() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());

    token_client.mint(&buyer, &10000);

    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);
    client.lock_funds();
    client.release_funds();

    // Releasing again should panic with InvalidState (Error 4)
    client.release_funds();
}

#[test]
fn test_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());
    let token_client_basic = soroban_sdk::token::Client::new(&env, &token_contract.address());

    // Mint tokens to buyer
    token_client.mint(&buyer, &10000);

    // Create and lock
    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);
    client.lock_funds();

    // Check balance before refund
    assert_eq!(token_client_basic.balance(&buyer), 9000);

    // Refund
    client.refund();

    // Check balances after refund
    assert_eq!(token_client_basic.balance(&contract_id), 0);
    assert_eq!(token_client_basic.balance(&buyer), 10000);

    env.as_contract(&contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_refund_already_released() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_contract.address());

    token_client.mint(&buyer, &10000);

    client.create_escrow(&buyer, &seller, &token_contract.address(), &amount);
    client.lock_funds();
    client.release_funds();

    // Try to refund after released
    client.refund();
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    // TODO: Set up environment, register contract, and mock tokens.
    // TODO: Lock funds first.
    // TODO: Call client.resolve_dispute(&mediator, &Symbol::new(&env, "refund_buyer")).
    // TODO: Assert that the funds were routed correctly based on the outcome.
}
