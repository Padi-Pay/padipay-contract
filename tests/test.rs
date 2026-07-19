#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, Symbol,
};

pub struct TestSetup<'a> {
    pub contract_id: Address,
    pub client: PadiPayEscrowContractClient<'a>,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub token_admin: Address,
    pub token_client: soroban_sdk::token::StellarAssetClient<'a>,
    pub token_client_basic: soroban_sdk::token::Client<'a>,
}

pub fn setup_test<'a>(env: &'a Env) -> TestSetup<'a> {
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);

    let buyer = Address::generate(env);
    let seller = Address::generate(env);

    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();
    let token_client = soroban_sdk::token::StellarAssetClient::new(env, &token);
    let token_client_basic = soroban_sdk::token::Client::new(env, &token);

    TestSetup {
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

#[test]
fn test_create_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowCreated"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
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
    let setup = setup_test(&env);
    let amount = 1000;

    // This should panic because buyer didn't authorize
    let _escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_create_escrow_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 0; // Invalid amount

    let _escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_create_escrow_invalid_addresses() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Buyer == seller
    let _escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.buyer, &setup.token, &amount);
}

#[test]
fn test_lock_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Mint tokens to buyer
    setup.token_client.mint(&setup.buyer, &10000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);

    // Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Lock funds
    setup.client.lock_funds(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "FundsLocked"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    // Check balances
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
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
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);

    // This should panic with AlreadyFunded
    setup.client.lock_funds(&escrow_id);
}

#[test]
fn test_release_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Mint tokens to buyer
    setup.token_client.mint(&setup.buyer, &10000);

    // Create escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Lock funds
    setup.client.lock_funds(&escrow_id);

    // Release funds
    setup.client.release_funds(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "FundsReleased"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    // Check balances
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 1000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
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
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);
    setup.client.release_funds(&escrow_id);

    // Releasing again should panic with InvalidState (Error 2)
    setup.client.release_funds(&escrow_id);
}

#[test]
fn test_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Mint tokens to buyer
    setup.token_client.mint(&setup.buyer, &10000);

    // Create and lock
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);

    // Check balance before refund
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);

    // Refund
    setup.client.refund(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowRefunded"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    // Check balances after refund
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
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
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);
    setup.client.release_funds(&escrow_id);

    // Try to refund after released
    setup.client.refund(&escrow_id);
}

#[test]
fn test_resolve_dispute_refund_buyer() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    // Mint tokens and lock funds
    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);

    // Verify balance before resolution
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);

    // Resolve dispute: refund_buyer
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "refund_buyer"));

    // Verify balance after resolution
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

    // Verify state is persisted and updated to Refunded
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
}

#[test]
fn test_resolve_dispute_pay_seller() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    // Mint tokens and lock funds
    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);

    // Verify balance before resolution
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

    // Resolve dispute: pay_seller
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    // Verify balance after resolution
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 1000);

    // Verify state is persisted and updated to Released
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
fn test_resolve_dispute_invalid_outcome() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;
    let mediator = Address::generate(&env);

    // Mint tokens and lock funds
    setup.token_client.mint(&setup.buyer, &10000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);
    setup.client.lock_funds(&escrow_id);

    // Resolve dispute with invalid outcome
    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "invalid_out"));
}

#[test]
fn test_escrow_lifecycle_happy_path_release() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 5000;

    // 1. Initial State
    setup.token_client.mint(&setup.buyer, &10000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);

    // 2. Create Escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowCreated"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(state.buyer, setup.buyer);
        assert_eq!(state.seller, setup.seller);
        assert_eq!(state.token, setup.token);
        assert_eq!(state.amount, amount);
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });

    // 3. Lock Funds
    setup.client.lock_funds(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "FundsLocked"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 5000);
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 5000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
    });

    // 4. Release Funds
    setup.client.release_funds(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "FundsReleased"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 5000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });
}

#[test]
fn test_escrow_lifecycle_happy_path_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 5000;

    // 1. Initial State
    setup.token_client.mint(&setup.buyer, &10000);

    // 2. Create Escrow
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Created
        );
    });

    // 3. Lock Funds
    setup.client.lock_funds(&escrow_id);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Locked
        );
    });

    // 4. Refund Funds
    setup.client.refund(&escrow_id);

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowRefunded"),
                    escrow_id,
                    setup.buyer.clone(),
                    setup.seller.clone()
                )
                    .into_val(&env),
                amount.into_val(&env)
            )
        ]
    );

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_lock_funds_unauthorized() {
    let escrow_id = 0;
    let env = Env::default();
    let setup = setup_test(&env);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: setup.buyer.clone(),
            seller: setup.seller.clone(),
            token: setup.token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Created,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, 0, &state);
    });

    setup.client.lock_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_release_funds_unauthorized() {
    let escrow_id = 0;
    let env = Env::default();
    let setup = setup_test(&env);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: setup.buyer.clone(),
            seller: setup.seller.clone(),
            token: setup.token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, 0, &state);
    });

    setup.client.release_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_refund_unauthorized() {
    let escrow_id = 0;
    let env = Env::default();
    let setup = setup_test(&env);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: setup.buyer.clone(),
            seller: setup.seller.clone(),
            token: setup.token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, 0, &state);
    });

    setup.client.refund(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_release_funds_invalid_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Try to release while still 'Created' (invalid state)
    setup.client.release_funds(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_refund_invalid_state() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount);

    // Try to refund while still 'Created' (invalid state)
    setup.client.refund(&escrow_id);
}

#[test]
fn test_multiple_concurrent_escrows() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    // Mint tokens for both
    setup.token_client.mint(&setup.buyer, &20000);

    // Create Escrow 1
    let amount1 = 1000;
    let escrow_id_1 =
        setup
            .client
            .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount1);

    // Create Escrow 2
    let amount2 = 5000;
    let escrow_id_2 =
        setup
            .client
            .create_escrow(&setup.buyer, &setup.seller, &setup.token, &amount2);

    // Validate unique IDs
    assert_eq!(escrow_id_1, 1);
    assert_eq!(escrow_id_2, 2);

    // Update Escrow 1 (Lock funds)
    setup.client.lock_funds(&escrow_id_1);

    // Read state 1
    let state1 = env.as_contract(&setup.contract_id, || {
        soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id_1).unwrap()
    });

    // Read state 2
    let state2 = env.as_contract(&setup.contract_id, || {
        soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id_2).unwrap()
    });

    // Validate Escrow 1 is locked, but Escrow 2 is still created
    assert_eq!(
        state1.status,
        soroban_escrow_contracts::types::EscrowStatus::Locked
    );
    assert_eq!(
        state2.status,
        soroban_escrow_contracts::types::EscrowStatus::Created
    );

    // Verify balances
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 19000); // 20000 - 1000 (locked) - 5000 (not locked yet, wait, create_escrow doesn't transfer funds). Wait, create_escrow DOES NOT transfer. Lock does. So balance is 20000 - 1000 = 19000.
}
