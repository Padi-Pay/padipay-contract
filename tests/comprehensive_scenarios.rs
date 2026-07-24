//! Comprehensive integration test scenarios for the PadiPay escrow contract.
#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{
    testutils::{
        storage::{Instance, Persistent},
        Address as _, Events,
    },
    vec, Address, Env, IntoVal, Symbol,
};

// =============================================================================
// Test Setup Helpers
// =============================================================================

/// Shared test setup struct for comprehensive integration tests.
pub struct ComprehensiveTestSetup<'a> {
    pub contract_id: Address,
    pub client: PadiPayEscrowContractClient<'a>,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub token_admin: Address,
    pub token_client: soroban_sdk::token::StellarAssetClient<'a>,
    pub token_client_basic: soroban_sdk::token::Client<'a>,
}

/// Creates a standard test environment with a contract, buyer, seller, and token.
pub fn comprehensive_setup<'a>(env: &'a Env) -> ComprehensiveTestSetup<'a> {
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);

    let buyer = Address::generate(env);
    let seller = Address::generate(env);

    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();
    let token_client = soroban_sdk::token::StellarAssetClient::new(env, &token);
    let token_client_basic = soroban_sdk::token::Client::new(env, &token);

    ComprehensiveTestSetup {
        contract_id,
        client,
        buyer,
        seller,
        token,
        token_admin,
        token_client,
        token_client_basic,
    }
}

/// Helper to create an escrow and return the escrow ID.
fn create_escrow_helper(
    setup: &ComprehensiveTestSetup,
    amount: i128,
) -> soroban_escrow_contracts::types::EscrowId {
    setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount)
}

/// Helper to create and lock an escrow. Returns the escrow ID.
fn create_and_lock_helper(
    setup: &ComprehensiveTestSetup,
    amount: i128,
) -> soroban_escrow_contracts::types::EscrowId {
    let escrow_id = create_escrow_helper(setup, amount);
    setup.client.lock_funds(&escrow_id);
    escrow_id
}

/// Helper to create, lock, and release an escrow. Returns the escrow ID.
fn create_lock_and_release_helper(
    setup: &ComprehensiveTestSetup,
    amount: i128,
) -> soroban_escrow_contracts::types::EscrowId {
    let escrow_id = create_and_lock_helper(setup, amount);
    setup.client.release_funds(&escrow_id);
    escrow_id
}

/// Helper to create, lock, and refund an escrow. Returns the escrow ID.
fn create_lock_and_refund_helper(
    setup: &ComprehensiveTestSetup,
    amount: i128,
) -> soroban_escrow_contracts::types::EscrowId {
    let escrow_id = create_and_lock_helper(setup, amount);
    setup.client.refund(&escrow_id);
    escrow_id
}

// =============================================================================
// Escrow Creation Tests
// =============================================================================

#[test]
fn test_create_escrow_minimum_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1; // Minimum valid amount

    let escrow_id = create_escrow_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
        assert_eq!(state.amount, 1);
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });
}

#[test]
fn test_create_escrow_large_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = i128::MAX / 2; // Very large amount

    let escrow_id = create_escrow_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(state.amount, amount);
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_create_escrow_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = -1;

    let _escrow_id = create_escrow_helper(&setup, amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_create_escrow_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 0;

    let _escrow_id = create_escrow_helper(&setup, amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_create_escrow_same_buyer_seller() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    // Using buyer as both buyer and seller should fail
    let _escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.buyer, &setup.token, &amount);
}

#[test]
fn test_create_multiple_escrows_sequential_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    let id1 = create_escrow_helper(&setup, 100);
    let id2 = create_escrow_helper(&setup, 200);
    let id3 = create_escrow_helper(&setup, 300);
    let id4 = create_escrow_helper(&setup, 400);
    let id5 = create_escrow_helper(&setup, 500);

    // IDs should be sequential starting from 1
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    assert_eq!(id4, 4);
    assert_eq!(id5, 5);
}

#[test]
fn test_create_escrow_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 2500;

    let escrow_id = create_escrow_helper(&setup, amount);

    let expected_state = soroban_escrow_contracts::types::EscrowState {
        buyer: setup.buyer.clone(),
        seller: setup.seller.clone(),
        token: setup.token.clone(),
        amount,
        status: soroban_escrow_contracts::types::EscrowStatus::Created,
    };
    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (Symbol::new(&env, "EscrowCreated"), escrow_id,).into_val(&env),
                expected_state.into_val(&env)
            )
        ]
    );
}

#[test]
fn test_create_escrow_stores_all_fields_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 9999;

    let escrow_id = create_escrow_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        // Verify every field
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
        assert_eq!(state.amount, 9999);
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });
}

#[test]
fn test_create_escrow_does_not_transfer_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    // Mint tokens to buyer
    setup.token_client.mint(&setup.buyer, &5000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 5000);

    // Create escrow (should NOT move any tokens)
    let _escrow_id = create_escrow_helper(&setup, amount);

    // Buyer balance should remain unchanged
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 5000);
    // Contract balance should be zero
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
}

// =============================================================================
// Lock Funds Tests
// =============================================================================

#[test]
fn test_lock_funds_transfers_exact_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 3000;

    setup.token_client.mint(&setup.buyer, &10000);
    let _escrow_id = create_and_lock_helper(&setup, amount);

    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 7000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 3000);
}

#[test]
fn test_lock_funds_updates_state_to_locked() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
        // Other fields should remain unchanged
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
        assert_eq!(state.amount, amount);
    });
}

#[test]
fn test_lock_funds_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1500;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    let expected_state = soroban_escrow_contracts::types::EscrowState {
        buyer: setup.buyer.clone(),
        seller: setup.seller.clone(),
        token: setup.token.clone(),
        amount,
        status: soroban_escrow_contracts::types::EscrowStatus::Locked,
    };
    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (Symbol::new(&env, "FundsLocked"), escrow_id,).into_val(&env),
                expected_state.into_val(&env)
            )
        ]
    );
}

#[test]
fn test_lock_funds_with_exact_buyer_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 5000;

    // Mint exactly the escrow amount
    setup.token_client.mint(&setup.buyer, &5000);
    let _escrow_id = create_and_lock_helper(&setup, amount);

    // Buyer should have zero balance after locking
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 5000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_lock_funds_when_already_locked() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Try to lock again - should fail
    setup.client.lock_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_lock_funds_nonexistent_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    // Try to lock a non-existent escrow
    setup.client.lock_funds(&999);
}

// =============================================================================
// Release Funds Tests
// =============================================================================

#[test]
fn test_release_funds_transfers_to_seller() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 2000;

    setup.token_client.mint(&setup.buyer, &10000);
    let _escrow_id = create_lock_and_release_helper(&setup, amount);

    // Seller should have received the funds
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 2000);
    // Contract should be empty
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    // Buyer should still have their remaining balance
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 8000);
}

#[test]
fn test_release_funds_updates_state_to_released() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_release_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });
}

#[test]
fn test_release_funds_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 4000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_release_helper(&setup, amount);

    let expected_state = soroban_escrow_contracts::types::EscrowState {
        buyer: setup.buyer.clone(),
        seller: setup.seller.clone(),
        token: setup.token.clone(),
        amount,
        status: soroban_escrow_contracts::types::EscrowStatus::Released,
    };
    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (Symbol::new(&env, "FundsReleased"), escrow_id,).into_val(&env),
                expected_state.into_val(&env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_release_funds_from_created_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    let escrow_id = create_escrow_helper(&setup, amount);

    // Try to release without locking first - should fail
    setup.client.release_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_release_funds_from_released_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_release_helper(&setup, amount);

    // Try to release again - should fail
    setup.client.release_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_release_funds_from_refunded_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_refund_helper(&setup, amount);

    // Try to release after refund - should fail
    setup.client.release_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_release_funds_nonexistent_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    setup.client.release_funds(&999);
}

// =============================================================================
// Refund Tests
// =============================================================================

#[test]
fn test_refund_returns_funds_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 3000;

    setup.token_client.mint(&setup.buyer, &10000);
    let _escrow_id = create_lock_and_refund_helper(&setup, amount);

    // Buyer should have their full balance back
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);
    // Contract should be empty
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    // Seller should have nothing
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);
}

#[test]
fn test_refund_updates_state_to_refunded() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_refund_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
}

#[test]
fn test_refund_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 7500;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_refund_helper(&setup, amount);

    let expected_state = soroban_escrow_contracts::types::EscrowState {
        buyer: setup.buyer.clone(),
        seller: setup.seller.clone(),
        token: setup.token.clone(),
        amount,
        status: soroban_escrow_contracts::types::EscrowStatus::Refunded,
    };
    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (Symbol::new(&env, "EscrowRefunded"), escrow_id,).into_val(&env),
                expected_state.into_val(&env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_refund_from_created_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    let escrow_id = create_escrow_helper(&setup, amount);

    // Try to refund without locking first - should fail
    setup.client.refund(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_refund_from_released_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_release_helper(&setup, amount);

    // Try to refund after release - should fail
    setup.client.refund(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_refund_from_refunded_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_refund_helper(&setup, amount);

    // Try to refund again - should fail
    setup.client.refund(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_refund_nonexistent_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    setup.client.refund(&999);
}

// =============================================================================
// Dispute Resolution Tests
// =============================================================================

#[test]
fn test_resolve_dispute_refund_buyer_balances() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 5000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Verify pre-dispute balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 5000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 5000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

    // Resolve: refund buyer
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "refund_buyer"));

    // Verify post-dispute balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);
}

#[test]
fn test_resolve_dispute_pay_seller_balances() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 5000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Resolve: pay seller
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    // Verify post-dispute balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 5000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 5000);
}

#[test]
fn test_resolve_dispute_refund_buyer_updates_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "refund_buyer"));

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
}

#[test]
fn test_resolve_dispute_pay_seller_updates_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });
}

#[test]
#[should_panic(expected = "Invalid outcome")]
fn test_resolve_dispute_with_empty_outcome() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Empty string as outcome
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, ""));
}

#[test]
#[should_panic(expected = "Invalid outcome")]
fn test_resolve_dispute_with_wrong_case_outcome() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Wrong case should be treated as invalid
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "Refund_Buyer"));
}

// =============================================================================
// Concurrent Escrow Isolation Tests
// =============================================================================

#[test]
fn test_multiple_escrows_independent_states() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    setup.token_client.mint(&setup.buyer, &100000);

    // Create 5 escrows with different amounts
    let id1 = create_escrow_helper(&setup, 1000);
    let id2 = create_escrow_helper(&setup, 2000);
    let id3 = create_escrow_helper(&setup, 3000);
    let id4 = create_escrow_helper(&setup, 4000);
    let id5 = create_escrow_helper(&setup, 5000);

    // Lock only escrows 1, 3, and 5
    setup.client.lock_funds(&id1);
    setup.client.lock_funds(&id3);
    setup.client.lock_funds(&id5);

    // Release escrow 1
    setup.client.release_funds(&id1);

    // Refund escrow 3
    setup.client.refund(&id3);

    // Verify each escrow has the correct independent state
    env.as_contract(&setup.contract_id, || {
        let state1 = soroban_escrow_contracts::storage::read_escrow_state(&env, id1).unwrap();
        assert_eq!(
            state1.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
        assert_eq!(state1.amount, 1000);

        let state2 = soroban_escrow_contracts::storage::read_escrow_state(&env, id2).unwrap();
        assert_eq!(
            state2.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
        assert_eq!(state2.amount, 2000);

        let state3 = soroban_escrow_contracts::storage::read_escrow_state(&env, id3).unwrap();
        assert_eq!(
            state3.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
        assert_eq!(state3.amount, 3000);

        let state4 = soroban_escrow_contracts::storage::read_escrow_state(&env, id4).unwrap();
        assert_eq!(
            state4.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
        assert_eq!(state4.amount, 4000);

        let state5 = soroban_escrow_contracts::storage::read_escrow_state(&env, id5).unwrap();
        assert_eq!(
            state5.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
        assert_eq!(state5.amount, 5000);
    });
}

#[test]
fn test_multiple_escrows_balance_accounting() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let initial_balance = 50000i128;

    setup.token_client.mint(&setup.buyer, &initial_balance);

    // Create and lock 3 escrows
    let _id1 = create_and_lock_helper(&setup, 5000);
    let _id2 = create_and_lock_helper(&setup, 10000);
    let _id3 = create_and_lock_helper(&setup, 15000);

    // Buyer: 50000 - 5000 - 10000 - 15000 = 20000
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 20000);
    // Contract: 5000 + 10000 + 15000 = 30000
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 30000);
}

#[test]
fn test_multiple_escrows_partial_release_and_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    setup.token_client.mint(&setup.buyer, &50000);

    let id1 = create_and_lock_helper(&setup, 10000);
    let id2 = create_and_lock_helper(&setup, 15000);
    let id3 = create_and_lock_helper(&setup, 5000);

    // Release escrow 1 (seller gets 10000)
    setup.client.release_funds(&id1);

    // Refund escrow 2 (buyer gets 15000 back)
    setup.client.refund(&id2);

    // Escrow 3 remains locked
    // Buyer: 50000 - 10000 - 15000 - 5000 + 15000 = 35000
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 35000);
    // Seller: 10000
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 10000);
    // Contract: 5000 (only escrow 3 still locked)
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 5000);

    // Now release escrow 3
    setup.client.release_funds(&id3);

    // Final balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 35000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 15000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
}

// =============================================================================
// TTL Extension Verification Tests
// =============================================================================

#[test]
fn test_create_escrow_extends_instance_and_persistent_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    let escrow_id = create_escrow_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let expected_ttl = 30 * 17_280;
        let persistent_ttl = env
            .storage()
            .persistent()
            .get_ttl(&soroban_escrow_contracts::types::DataKey::Escrow(escrow_id));
        assert_eq!(persistent_ttl, expected_ttl);

        let instance_ttl = env.storage().instance().get_ttl();
        assert_eq!(instance_ttl, expected_ttl);
    });
}

#[test]
fn test_lock_funds_extends_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let expected_ttl = 30 * 17_280;
        let persistent_ttl = env
            .storage()
            .persistent()
            .get_ttl(&soroban_escrow_contracts::types::DataKey::Escrow(escrow_id));
        assert_eq!(persistent_ttl, expected_ttl);

        let instance_ttl = env.storage().instance().get_ttl();
        assert_eq!(instance_ttl, expected_ttl);
    });
}

#[test]
fn test_release_funds_extends_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_release_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let expected_ttl = 30 * 17_280;
        let persistent_ttl = env
            .storage()
            .persistent()
            .get_ttl(&soroban_escrow_contracts::types::DataKey::Escrow(escrow_id));
        assert_eq!(persistent_ttl, expected_ttl);

        let instance_ttl = env.storage().instance().get_ttl();
        assert_eq!(instance_ttl, expected_ttl);
    });
}

#[test]
fn test_refund_extends_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_lock_and_refund_helper(&setup, amount);

    env.as_contract(&setup.contract_id, || {
        let expected_ttl = 30 * 17_280;
        let persistent_ttl = env
            .storage()
            .persistent()
            .get_ttl(&soroban_escrow_contracts::types::DataKey::Escrow(escrow_id));
        assert_eq!(persistent_ttl, expected_ttl);

        let instance_ttl = env.storage().instance().get_ttl();
        assert_eq!(instance_ttl, expected_ttl);
    });
}

#[test]
fn test_resolve_dispute_extends_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = create_and_lock_helper(&setup, amount);

    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    env.as_contract(&setup.contract_id, || {
        let expected_ttl = 30 * 17_280;
        let persistent_ttl = env
            .storage()
            .persistent()
            .get_ttl(&soroban_escrow_contracts::types::DataKey::Escrow(escrow_id));
        assert_eq!(persistent_ttl, expected_ttl);

        let instance_ttl = env.storage().instance().get_ttl();
        assert_eq!(instance_ttl, expected_ttl);
    });
}

// =============================================================================
// State Transition Matrix Validation
// =============================================================================

#[test]
fn test_valid_transition_created_to_locked() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Created;
    assert!(status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Locked));
}

#[test]
fn test_valid_transition_locked_to_released() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Locked;
    assert!(status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Released));
}

#[test]
fn test_valid_transition_locked_to_refunded() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Locked;
    assert!(status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Refunded));
}

#[test]
fn test_invalid_transition_created_to_released() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Created;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Released));
}

#[test]
fn test_invalid_transition_created_to_refunded() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Created;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Refunded));
}

#[test]
fn test_invalid_transition_created_to_created() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Created;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Created));
}

#[test]
fn test_invalid_transition_locked_to_locked() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Locked;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Locked));
}

#[test]
fn test_invalid_transition_locked_to_created() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Locked;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Created));
}

#[test]
fn test_invalid_transition_released_to_any() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Released;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Created));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Locked));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Released));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Refunded));
}

#[test]
fn test_invalid_transition_refunded_to_any() {
    let status = soroban_escrow_contracts::types::EscrowStatus::Refunded;
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Created));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Locked));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Released));
    assert!(!status.is_valid_transition(&soroban_escrow_contracts::types::EscrowStatus::Refunded));
}

// =============================================================================
// Full Lifecycle Integration Tests
// =============================================================================

#[test]
fn test_full_lifecycle_create_lock_release_with_all_verifications() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 8000;

    // Setup: Mint tokens
    setup.token_client.mint(&setup.buyer, &20000);

    // Step 1: Create escrow
    let escrow_id = create_escrow_helper(&setup, amount);

    // Verify state after creation
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
        assert_eq!(state.amount, 8000);
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
    });

    // Verify no tokens moved
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 20000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);

    // Step 2: Lock funds
    setup.client.lock_funds(&escrow_id);

    // Verify state after locking
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
    });

    // Verify tokens moved to contract
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 12000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 8000);

    // Step 3: Release funds
    setup.client.release_funds(&escrow_id);

    // Verify state after release
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });

    // Verify final balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 12000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 8000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
}

#[test]
fn test_full_lifecycle_create_lock_refund_with_all_verifications() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 6000;

    // Setup: Mint tokens
    setup.token_client.mint(&setup.buyer, &20000);

    // Step 1: Create escrow
    let escrow_id = create_escrow_helper(&setup, amount);

    // Verify state after creation
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });

    // Step 2: Lock funds
    setup.client.lock_funds(&escrow_id);

    // Verify state after locking
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
    });

    // Verify tokens moved to contract
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 14000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 6000);

    // Step 3: Refund
    setup.client.refund(&escrow_id);

    // Verify state after refund
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });

    // Verify final balances - buyer gets funds back
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 20000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
}

#[test]
fn test_full_lifecycle_with_dispute_resolution() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let amount = 10000;
    let mediator = Address::generate(&env);

    // Setup
    setup.token_client.mint(&setup.buyer, &25000);

    // Create and lock
    let escrow_id = create_and_lock_helper(&setup, amount);

    // Verify locked state
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 15000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 10000);

    // Resolve dispute in favor of seller
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    // Verify final state
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });

    // Verify final balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 15000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 10000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
}

// =============================================================================
// Nonce / ID Generation Tests
// =============================================================================

#[test]
fn test_nonce_increments_across_different_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);

    setup.token_client.mint(&setup.buyer, &100000);

    // Each create_escrow call should produce a unique, incrementing ID
    let id1 = create_escrow_helper(&setup, 100);
    let id2 = create_escrow_helper(&setup, 200);

    // Lock and release the first one
    setup.client.lock_funds(&id1);
    setup.client.release_funds(&id1);

    // Creating another escrow should continue the sequence
    let id3 = create_escrow_helper(&setup, 300);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

// =============================================================================
// Edge Case: Different Token Contracts
// =============================================================================

#[test]
fn test_escrow_with_different_token_contracts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Create two different tokens
    let token_admin1 = Address::generate(&env);
    let token_contract1 = env.register_stellar_asset_contract_v2(token_admin1.clone());
    let token1 = token_contract1.address();
    let token_client1 = soroban_sdk::token::StellarAssetClient::new(&env, &token1);
    let token_client1_basic = soroban_sdk::token::Client::new(&env, &token1);

    let token_admin2 = Address::generate(&env);
    let token_contract2 = env.register_stellar_asset_contract_v2(token_admin2.clone());
    let token2 = token_contract2.address();
    let token_client2 = soroban_sdk::token::StellarAssetClient::new(&env, &token2);
    let token_client2_basic = soroban_sdk::token::Client::new(&env, &token2);

    // Mint different tokens
    token_client1.mint(&buyer, &10000);
    token_client2.mint(&buyer, &20000);

    // Create escrows with different tokens
    let id1 = client.create_escrow(&buyer, &seller, &token1, &5000);
    let id2 = client.create_escrow(&buyer, &seller, &token2, &8000);

    // Lock both
    client.lock_funds(&id1);
    client.lock_funds(&id2);

    // Verify token1 balances
    assert_eq!(token_client1_basic.balance(&buyer), 5000);
    assert_eq!(token_client1_basic.balance(&contract_id), 5000);

    // Verify token2 balances
    assert_eq!(token_client2_basic.balance(&buyer), 12000);
    assert_eq!(token_client2_basic.balance(&contract_id), 8000);

    // Release escrow 1 (token1 goes to seller)
    client.release_funds(&id1);
    assert_eq!(token_client1_basic.balance(&seller), 5000);
    assert_eq!(token_client1_basic.balance(&contract_id), 0);

    // Refund escrow 2 (token2 goes back to buyer)
    client.refund(&id2);
    assert_eq!(token_client2_basic.balance(&buyer), 20000);
    assert_eq!(token_client2_basic.balance(&contract_id), 0);
}
// =============================================================================
// Additional Comprehensive Verification Checks
// =============================================================================
#[test]
fn test_other_user_balance_is_not_affected() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = comprehensive_setup(&env);
    let other_user = Address::generate(&env);
    setup.token_client.mint(&other_user, &100);
    setup.token_client.mint(&setup.buyer, &100);
    let id = create_and_lock_helper(&setup, 50);
    setup.client.release_funds(&id);
    // Verify that the other user's balance remains unchanged
    assert_eq!(setup.token_client_basic.balance(&other_user), 100);
}
