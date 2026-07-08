#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

#[test]
fn test_create_escrow() {
    let env = Env::default();
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
fn test_lock_funds() {
    let env = Env::default();
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000;

    // TODO: Mock token contract and mint initial balance to the buyer.
    // TODO: Call client.lock_funds(&buyer, &seller, &amount).
    // TODO: Assert that the contract holds the tokens and buyer's balance decreased.
}

#[test]
fn test_release_funds() {
    let env = Env::default();
    // TODO: Set up environment, register contract, and mock tokens.
    // TODO: Lock funds first.
    // TODO: Call client.release_funds(&buyer).
    // TODO: Assert that the seller received the funds.
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    // TODO: Set up environment, register contract, and mock tokens.
    // TODO: Lock funds first.
    // TODO: Call client.resolve_dispute(&mediator, &Symbol::new(&env, "refund_buyer")).
    // TODO: Assert that the funds were routed correctly based on the outcome.
}
