//! Integration scenarios and validation test suite for the token abstraction wrapper.
#![cfg(test)]

use soroban_escrow_contracts::token::{
    allowance, approve, balance, get_token_client, transfer, transfer_from,
};
use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

/// Structured setup for token abstraction integration testing.
pub struct TokenTestSetup<'a> {
    pub env: Env,
    pub contract_id: Address,
    pub client: PadiPayEscrowContractClient<'a>,
    pub token: Address,
    pub token_admin: Address,
    pub token_client: soroban_sdk::token::StellarAssetClient<'a>,
    pub token_client_basic: soroban_sdk::token::Client<'a>,
    pub buyer: Address,
    pub seller: Address,
    pub mediator: Address,
    pub other_user: Address,
}

/// Initializes the test environment, registers the escrow contract, mock token, and pre-funds accounts.
pub fn setup_token_test<'a>(env: &Env) -> TokenTestSetup<'a> {
    env.mock_all_auths();

    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();
    let token_client = soroban_sdk::token::StellarAssetClient::new(env, &token);
    let token_client_basic = soroban_sdk::token::Client::new(env, &token);

    let buyer = Address::generate(env);
    let seller = Address::generate(env);
    let mediator = Address::generate(env);
    let other_user = Address::generate(env);

    // Fund the accounts with initial tokens
    token_client.mint(&buyer, &10_000);
    token_client.mint(&seller, &5_000);
    token_client.mint(&other_user, &2_000);

    TokenTestSetup {
        env: env.clone(),
        contract_id,
        client,
        token,
        token_admin,
        token_client,
        token_client_basic,
        buyer,
        seller,
        mediator,
        other_user,
    }
}

// =========================================================================
// 1. CLIENT INITIALIZATION & METADATA SCENARIOS
// =========================================================================

#[test]
fn test_get_token_client_address_matching() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let client = get_token_client(&env, &setup.token);
    assert_eq!(client.address, setup.token);
}

#[test]
fn test_get_token_client_distinct_addresses() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let another_token_admin = Address::generate(&env);
    let another_token_contract = env.register_stellar_asset_contract_v2(another_token_admin);
    let another_token = another_token_contract.address();

    let client_a = get_token_client(&env, &setup.token);
    let client_b = get_token_client(&env, &another_token);

    assert_ne!(client_a.address, client_b.address);
}

// =========================================================================
// 2. BALANCE RETRIEVAl TESTS
// =========================================================================

#[test]
fn test_balance_retrieval_correctness() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 10_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 5_000);
    assert_eq!(balance(&env, &setup.token, &setup.other_user), 2_000);
}

#[test]
fn test_balance_retrieval_empty_account() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let empty_address = Address::generate(&env);
    assert_eq!(balance(&env, &setup.token, &empty_address), 0);
}

#[test]
fn test_balance_retrieval_after_direct_mint() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 10_000);
    setup.token_client.mint(&setup.buyer, &1_500);
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 11_500);
}

// =========================================================================
// 3. ALLOWANCE AND APPROVE SCENARIOS
// =========================================================================

#[test]
fn test_allowance_default_is_zero() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let current_allowance = allowance(&env, &setup.token, &setup.buyer, &setup.seller);
    assert_eq!(current_allowance, 0);
}

#[test]
fn test_approve_sets_allowance_correctly() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 500;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &amount,
        expiration,
    );

    let current_allowance = allowance(&env, &setup.token, &setup.buyer, &setup.seller);
    assert_eq!(current_allowance, amount);
}

#[test]
fn test_approve_overwrite_existing_allowance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount_1 = 500;
    let amount_2 = 1_200;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &amount_1,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        amount_1
    );

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &amount_2,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        amount_2
    );
}

#[test]
fn test_approve_zero_clears_allowance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 1_000;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &amount,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        amount
    );

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &0,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        0
    );
}

#[test]
#[should_panic(expected = "amount must be non-negative")]
fn test_approve_negative_amount_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let negative_amount = -100;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &negative_amount,
        expiration,
    );
}

#[test]
fn test_approve_multiple_spenders_isolation() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let spender_a = Address::generate(&env);
    let spender_b = Address::generate(&env);

    approve(&env, &setup.token, &setup.buyer, &spender_a, &500, 100);
    approve(&env, &setup.token, &setup.buyer, &spender_b, &1_500, 100);

    assert_eq!(allowance(&env, &setup.token, &setup.buyer, &spender_a), 500);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &spender_b),
        1_500
    );
}

// =========================================================================
// 4. TRANSFER SCENARIOS
// =========================================================================

#[test]
fn test_transfer_happy_path() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 2_000;

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &amount);

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 8_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 7_000);
}

#[test]
#[should_panic]
fn test_transfer_insufficient_funds_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Buyer has 10_000, trying to transfer 10_001
    let amount = 10_001;

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &amount);
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_zero_amount_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &0);
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_negative_amount_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let negative_amount = -500;
    transfer(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &negative_amount,
    );
}

#[test]
#[should_panic(expected = "sender and recipient addresses must be different")]
fn test_transfer_self_transfer_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 1_000;
    transfer(&env, &setup.token, &setup.buyer, &setup.buyer, &amount);
}

// =========================================================================
// 5. TRANSFER_FROM SCENARIOS
// =========================================================================

#[test]
fn test_transfer_from_happy_path() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 3_000;
    let transfer_amount = 1_500;
    let expiration = 100;

    // Approve the spender (seller) to spend on behalf of buyer
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &allowance_amount,
        expiration,
    );

    // Spender (seller) transfers tokens from buyer to seller
    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.seller,
        &transfer_amount,
    );

    // Verify balances
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 8_500);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 6_500);

    // Verify allowance is decremented
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        1_500
    );
}

#[test]
#[should_panic]
fn test_transfer_from_exceeds_allowance_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 1_000;
    let transfer_amount = 1_001;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &allowance_amount,
        expiration,
    );

    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.seller,
        &transfer_amount,
    );
}

#[test]
#[should_panic]
fn test_transfer_from_exceeds_balance_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Buyer has 10_000 tokens, approve 15_000 spending
    let allowance_amount = 15_000;
    let transfer_amount = 12_000;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &allowance_amount,
        expiration,
    );

    // This should panic due to insufficient buyer balance, even if allowance is OK
    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.seller,
        &transfer_amount,
    );
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_from_zero_amount_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.seller,
        &0,
    );
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_from_negative_amount_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let negative_amount = -200;
    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.seller,
        &negative_amount,
    );
}

#[test]
#[should_panic(expected = "sender and recipient addresses must be different")]
fn test_transfer_from_self_transfer_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 500;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &amount,
        100,
    );

    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.buyer,
        &amount,
    );
}

// =========================================================================
// 6. COMPLEX SEQUENTIAL TRANSACTION SCENARIOS
// =========================================================================

#[test]
fn test_allowance_stepwise_drain() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 5_000;
    let expiration = 100;

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        expiration,
    );

    // Step 1: Spend 1,000
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &1_000,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        4_000
    );

    // Step 2: Spend 2,500
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &2_500,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        1_500
    );

    // Step 3: Spend remaining 1,500
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &1_500,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        0
    );

    // Verify final balances
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 5_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 10_000);
}

#[test]
fn test_allowance_increase_and_decrease() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let expiration = 100;

    // Start with 1,000 allowance
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &1_000,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        1_000
    );

    // Increase to 2,500
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &2_500,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        2_500
    );

    // Spend 1,000
    transfer_from(
        &env,
        &setup.token,
        &setup.seller,
        &setup.buyer,
        &setup.other_user,
        &1_000,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        1_500
    );

    // Override remaining allowance to 500
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &500,
        expiration,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.seller),
        500
    );
}

// =========================================================================
// 7. CONTRACT LIFE CYCLE WITH WRAPPED TOKENS
// =========================================================================

#[test]
fn test_escrow_complete_happy_flow_using_wrappers() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 4_000;

    // Step 1: Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Step 2: Lock funds
    // Approve contract address to pull funds from buyer
    let contract_as_spender = &setup.contract_id;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        contract_as_spender,
        &amount,
        100,
    );

    // Perform Lock
    setup.client.lock_funds(&escrow_id);

    // Verify balances after locking
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 6_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), amount);

    // Step 3: Release funds
    setup.client.release_funds(&escrow_id);

    // Verify final balances after release
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 6_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 9_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), 0);
}

#[test]
fn test_escrow_complete_refund_flow_using_wrappers() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 3_000;

    // Step 1: Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Step 2: Lock funds
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.contract_id,
        &amount,
        100,
    );
    setup.client.lock_funds(&escrow_id);

    // Verify balances after lock
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 7_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), amount);

    // Step 3: Refund
    setup.client.refund(&escrow_id);

    // Verify final balances after refund
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 10_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), 0);
}

#[test]
fn test_escrow_dispute_pay_seller_flow_using_wrappers() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 5_000;

    // Step 1: Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Step 2: Lock funds
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.contract_id,
        &amount,
        100,
    );
    setup.client.lock_funds(&escrow_id);

    // Step 3: Dispute resolved in favor of pay_seller
    let outcome = Symbol::new(&env, "pay_seller");
    setup
        .client
        .resolve_dispute(&escrow_id, &setup.mediator, &outcome);

    // Verify balances after resolve
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 5_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 10_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), 0);
}

#[test]
fn test_escrow_dispute_refund_buyer_flow_using_wrappers() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 2_500;

    // Step 1: Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Step 2: Lock funds
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.contract_id,
        &amount,
        100,
    );
    setup.client.lock_funds(&escrow_id);

    // Step 3: Dispute resolved in favor of refund_buyer
    let outcome = Symbol::new(&env, "refund_buyer");
    setup
        .client
        .resolve_dispute(&escrow_id, &setup.mediator, &outcome);

    // Verify balances after resolve
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 10_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), 0);
}

// =========================================================================
// 8. CORNER CASES AND ROBUSTNESS
// =========================================================================

#[test]
fn test_multiple_interleaved_approvals_and_transfers() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let spender = &setup.other_user;

    approve(&env, &setup.token, &setup.buyer, spender, &5_000, 200);
    assert_eq!(allowance(&env, &setup.token, &setup.buyer, spender), 5_000);

    // Spend 2,000
    transfer_from(
        &env,
        &setup.token,
        spender,
        &setup.buyer,
        &setup.seller,
        &2_000,
    );
    assert_eq!(allowance(&env, &setup.token, &setup.buyer, spender), 3_000);

    // Increase approval again by writing a new total of 6_000
    approve(&env, &setup.token, &setup.buyer, spender, &6_000, 200);
    assert_eq!(allowance(&env, &setup.token, &setup.buyer, spender), 6_000);

    // Spend 5_000
    transfer_from(
        &env,
        &setup.token,
        spender,
        &setup.buyer,
        &setup.seller,
        &5_000,
    );
    assert_eq!(allowance(&env, &setup.token, &setup.buyer, spender), 1_000);

    // Try to overwrite allowance to 0
    approve(&env, &setup.token, &setup.buyer, spender, &0, 200);
    assert_eq!(allowance(&env, &setup.token, &setup.buyer, spender), 0);
}

#[test]
fn test_token_transfers_do_not_interfere_with_other_tokens() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Register a second mock token contract
    let secondary_admin = Address::generate(&env);
    let secondary_token_contract = env.register_stellar_asset_contract_v2(secondary_admin.clone());
    let secondary_token = secondary_token_contract.address();
    let secondary_token_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &secondary_token);

    // Mint second token to buyer
    secondary_token_client.mint(&setup.buyer, &3_000);

    // Check balances
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 10_000);
    assert_eq!(balance(&env, &secondary_token, &setup.buyer), 3_000);

    // Transfer primary token
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &1_000);

    // Primary token balance should change, secondary token balance should NOT change
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 9_000);
    assert_eq!(balance(&env, &secondary_token, &setup.buyer), 3_000);
}

// =========================================================================
// 9. ADDITIONAL EDGE CASES AND BOUNDARY CONDITIONS
// =========================================================================

#[test]
fn test_allowance_expiration_ledger_behavior() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 2_000;
    let expiration_ledger = 50;

    // Set initial ledger sequence to 10
    let mut info = env.ledger().get();
    info.sequence_number = 10;
    env.ledger().set(info.clone());

    // Approve the spender
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        expiration_ledger,
    );

    // Verify allowance is active
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        allowance_amount
    );

    // Spender can transfer when ledger sequence is 49 (before expiration)
    info.sequence_number = 49;
    env.ledger().set(info.clone());

    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        allowance_amount
    );

    // Move ledger past expiration (ledger sequence 51)
    info.sequence_number = 51;
    env.ledger().set(info);

    // Allowance should now be expired (returns 0)
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        0
    );
}

#[test]
fn test_approve_exceeds_available_balance_succeeds() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Buyer has 10_000 tokens, but we approve 50_000
    let allowance_amount = 50_000;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        100,
    );

    // Allowance should be successfully set to 50_000
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        allowance_amount
    );

    // Mint more tokens to buyer so they have 20_000 tokens
    setup.token_client.mint(&setup.buyer, &10_000);

    // Now transferring 12_000 should succeed since balance is sufficient
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &12_000,
    );

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 8_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 17_000);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        38_000
    );
}

#[test]
#[should_panic]
fn test_transfer_exceeds_available_balance_fails() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Buyer has 10_000 tokens, approve 50_000
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &50_000,
        100,
    );

    // Transferring 12_000 should panic because buyer only has 10_000
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &12_000,
    );
}

#[test]
#[should_panic]
fn test_transfer_with_extremely_large_amount() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Attempting to transfer maximum possible integer
    let giant_amount = i128::MAX;
    transfer(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &giant_amount,
    );
}

#[test]
fn test_transfer_boundary_exactly_balance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let buyer_balance = balance(&env, &setup.token, &setup.buyer);
    assert_eq!(buyer_balance, 10_000);

    // Transfer exact balance
    transfer(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.seller,
        &buyer_balance,
    );

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 0);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 15_000);
}

#[test]
fn test_transfer_boundary_exactly_one_token() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &1);

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 9_999);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 5_001);
}

#[test]
fn test_transfer_from_boundary_exactly_allowance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 3_000;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        100,
    );

    // Spend exactly the allowance
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &allowance_amount,
    );

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 7_000);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 8_000);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        0
    );
}

#[test]
fn test_transfer_from_boundary_exactly_balance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Allowance is larger than balance
    let allowance_amount = 15_000;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        100,
    );

    // Transfer exactly the balance (10_000)
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &10_000,
    );

    assert_eq!(balance(&env, &setup.token, &setup.buyer), 0);
    assert_eq!(balance(&env, &setup.token, &setup.seller), 15_000);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        5_000
    );
}

#[test]
fn test_approve_large_value_numeric_limits() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Approve the maximum possible i128 value
    let max_allowance = i128::MAX;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &max_allowance,
        200,
    );

    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        max_allowance
    );
}

#[test]
fn test_sequential_transfers_drain_to_zero() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Send 5000, then 3000, then 2000
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &5_000);
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 5_000);

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &3_000);
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 2_000);

    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &2_000);
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 0);
}

#[test]
#[should_panic]
fn test_transfer_when_drained_fails() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    // Drain balance to 0
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &10_000);

    // Next transfer of 1 should fail
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &1);
}

#[test]
fn test_allowance_expiration_exact_ledger_boundary() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let allowance_amount = 1_000;
    let expiration_ledger = 50;

    // Set initial ledger sequence to 10
    let mut info = env.ledger().get();
    info.sequence_number = 10;
    env.ledger().set(info.clone());

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &allowance_amount,
        expiration_ledger,
    );

    // Sequence 49: Spender is still authorized
    info.sequence_number = 49;
    env.ledger().set(info.clone());
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        allowance_amount
    );

    // Sequence 50: Expiration ledger boundary. Allowance is still valid at sequence == expiration_ledger.
    info.sequence_number = 50;
    env.ledger().set(info.clone());
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        allowance_amount
    );

    // Sequence 51: Past expiration ledger. Allowance is expired when sequence > expiration_ledger.
    info.sequence_number = 51;
    env.ledger().set(info);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        0
    );
}

#[test]
fn test_approve_exceeds_available_balance_and_multiple_spenders() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let spender_a = Address::generate(&env);
    let spender_b = Address::generate(&env);

    // Buyer has 10_000 tokens.
    // Approve Spender A for 8_000, Spender B for 8_000 (total approved = 16_000 > balance).
    approve(&env, &setup.token, &setup.buyer, &spender_a, &8_000, 100);
    approve(&env, &setup.token, &setup.buyer, &spender_b, &8_000, 100);

    // Spender A transfers 7_000 first (succeeds)
    transfer_from(
        &env,
        &setup.token,
        &spender_a,
        &setup.buyer,
        &setup.seller,
        &7_000,
    );
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 3_000);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &spender_a),
        1_000
    );

    // Spender B transfers 3_000 (succeeds, draining buyer to 0)
    transfer_from(
        &env,
        &setup.token,
        &spender_b,
        &setup.buyer,
        &setup.seller,
        &3_000,
    );
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 0);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &spender_b),
        5_000
    );
}

#[test]
#[should_panic]
fn test_multiple_spenders_empty_balance_fails() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let spender_a = Address::generate(&env);
    let spender_b = Address::generate(&env);

    approve(&env, &setup.token, &setup.buyer, &spender_a, &8_000, 100);
    approve(&env, &setup.token, &setup.buyer, &spender_b, &8_000, 100);

    // Spender A transfers 7_000 first (succeeds)
    transfer_from(
        &env,
        &setup.token,
        &spender_a,
        &setup.buyer,
        &setup.seller,
        &7_000,
    );

    // Spender B tries to transfer 7_000 (fails because balance is only 3_000)
    transfer_from(
        &env,
        &setup.token,
        &spender_b,
        &setup.buyer,
        &setup.seller,
        &7_000,
    );
}

#[test]
fn test_mediator_escrow_dispute_resolution_with_allowance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    let amount = 5_000;

    // Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Lock funds using allowance to contract
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.contract_id,
        &amount,
        100,
    );
    setup.client.lock_funds(&escrow_id);

    // Verify buyer balance dropped to 5_000
    assert_eq!(balance(&env, &setup.token, &setup.buyer), 5_000);

    // Dispute resolved by mediator paying the seller
    setup.client.resolve_dispute(
        &escrow_id,
        &setup.mediator,
        &Symbol::new(&env, "pay_seller"),
    );

    // Verify balances
    assert_eq!(balance(&env, &setup.token, &setup.seller), 10_000);
    assert_eq!(balance(&env, &setup.token, &setup.contract_id), 0);
}

#[test]
#[should_panic(expected = "sender and recipient addresses must be different")]
fn test_direct_self_transfer_fails() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    transfer(&env, &setup.token, &setup.buyer, &setup.buyer, &500);
}

#[test]
#[should_panic(expected = "sender and recipient addresses must be different")]
fn test_allowance_self_transfer_fails() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    let amount = 500;
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &amount,
        100,
    );
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.buyer,
        &amount,
    );
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_direct_transfer_zero_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &0);
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_direct_transfer_negative_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    transfer(&env, &setup.token, &setup.buyer, &setup.seller, &-10);
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_from_zero_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &500,
        100,
    );
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &0,
    );
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_from_negative_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &500,
        100,
    );
    transfer_from(
        &env,
        &setup.token,
        &setup.other_user,
        &setup.buyer,
        &setup.seller,
        &-10,
    );
}

#[test]
#[should_panic(expected = "amount must be non-negative")]
fn test_approve_negative_value_panics() {
    let env = Env::default();
    let setup = setup_token_test(&env);
    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &-5,
        100,
    );
}

#[test]
fn test_approve_zero_value_clears_allowance() {
    let env = Env::default();
    let setup = setup_token_test(&env);

    approve(
        &env,
        &setup.token,
        &setup.buyer,
        &setup.other_user,
        &500,
        100,
    );
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        500
    );

    approve(&env, &setup.token, &setup.buyer, &setup.other_user, &0, 100);
    assert_eq!(
        allowance(&env, &setup.token, &setup.buyer, &setup.other_user),
        0
    );
}
