#![cfg(test)]

use soroban_escrow_contracts::{
    events::EVENT_SCHEMA_VERSION, PadiPayEscrowContract, PadiPayEscrowContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowCreated"),
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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

/// Every lifecycle event must carry the schema version (`v1`) as its second
/// topic, directly after the event name, so off-chain indexers can filter
/// events by the payload shape they understand.
#[test]
fn test_events_carry_schema_version() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &amount);

    // The schema version symbol is the second topic of every event, directly
    // after the event name, so an indexer can subscribe with a
    // `(event_name, version)` topic filter.
    let v = Symbol::new(&env, EVENT_SCHEMA_VERSION);
    let expect_versioned_event = |name: &str, escrow_id: u64| {
        assert_eq!(
            env.events().all().filter_by_contract(&setup.contract_id),
            vec![
                &env,
                (
                    setup.contract_id.clone(),
                    (
                        Symbol::new(&env, name),
                        v.clone(),
                        escrow_id,
                        setup.buyer.clone(),
                        setup.seller.clone()
                    )
                        .into_val(&env),
                    amount.into_val(&env)
                )
            ]
        );
    };

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
    expect_versioned_event("EscrowCreated", escrow_id);

    setup.client.lock_funds(&escrow_id);
    expect_versioned_event("FundsLocked", escrow_id);

    setup.client.release_funds(&escrow_id);
    expect_versioned_event("FundsReleased", escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_create_escrow_unauthorized() {
    let env = Env::default();
    let setup = setup_test(&env);
    let amount = 1000;

    // This should panic because buyer didn't authorize
    let _escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_create_escrow_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 0; // Invalid amount

    let _escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_create_escrow_invalid_addresses() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Buyer == seller
    let _escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.buyer,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
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
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
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
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
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
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );
    setup.client.lock_funds(&escrow_id);
    setup.client.release_funds(&escrow_id);

    // Try to refund after released
    setup.client.refund(&escrow_id);
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    // Mint tokens to buyer
    setup.token_client.mint(&setup.buyer, &10000);

    // Create and lock
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    // Initial balances after lock
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

    // Resolve dispute in favor of seller
    setup
        .client
        .resolve_dispute(&escrow_id, &Symbol::new(&env, "pay_seller"));

    // Verify seller received funds
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 1000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Released
        );
    });

    // Test dispute in favor of buyer
    let amount2 = 2000;
    let escrow_id_2 = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount2,
        &0,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id_2);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 2000);

    // Resolve dispute in favor of buyer
    setup
        .client
        .resolve_dispute(&escrow_id_2, &Symbol::new(&env, "refund_buyer"));

    // Verify buyer received funds
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000); // Because they started with 10k, spent 1k on first escrow (released to seller), and second escrow was 2k but refunded. So 10k - 1k = 9k.

    env.as_contract(&setup.contract_id, || {
        let state =
            soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id_2).unwrap();
        assert_eq!(
            state.status,
            soroban_escrow_contracts::types::EscrowStatus::Refunded
        );
    });
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
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

    let events = env.events().all().filter_by_contract(&setup.contract_id);
    assert_eq!(
        events,
        vec![
            &env,
            (
                setup.contract_id.clone(),
                (
                    Symbol::new(&env, "EscrowCreated"),
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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
    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

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
                    Symbol::new(&env, EVENT_SCHEMA_VERSION),
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
            deadline: 0,
            mediator: setup.token_admin.clone(),
            timeout_ledger: None,
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
            deadline: 0,
            mediator: setup.token_admin.clone(),
            timeout_ledger: None,
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
            deadline: 0,
            mediator: setup.token_admin.clone(),
            timeout_ledger: None,
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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

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

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

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
    let escrow_id_1 = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount1,
        &0,
        &setup.token_admin,
        &None,
    );

    // Create Escrow 2
    let amount2 = 5000;
    let escrow_id_2 = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount2,
        &0,
        &setup.token_admin,
        &None,
    );

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

#[test]
fn test_execute_timeout_success() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    // Set deadline to 100 seconds from now
    let current_time = env.ledger().timestamp();
    let deadline = current_time + 100;

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &deadline,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);

    // Advance ledger time past deadline
    env.ledger().set_timestamp(deadline + 1);

    setup.client.execute_timeout(&escrow_id);

    // Verify refund
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_execute_timeout_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    let current_time = env.ledger().timestamp();
    let deadline = current_time + 100;

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &deadline,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    // Try to timeout before deadline
    setup.client.execute_timeout(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_execute_timeout_after_release() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);
    let current_time = env.ledger().timestamp();
    let deadline = current_time + 100;

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &deadline,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id);
    setup.client.release_funds(&escrow_id);

    // Advance ledger time past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should fail with InvalidState because it's already Released
    setup.client.execute_timeout(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_resolve_dispute_unauthorized() {
    let env = Env::default();
    let setup = setup_test(&env);
    let _amount = 1000;

    // Use a contract wrapper to bypass standard auth mocking so it actually fails
    let escrow_id = 0; // We mock state instead of calling create_escrow which requires auth

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: setup.buyer.clone(),
            seller: setup.seller.clone(),
            token: setup.token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
            deadline: 0,
            mediator: setup.token_admin.clone(),
            timeout_ledger: None,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, 0, &state);
    });

    // Try to resolve dispute without mediator auth
    setup
        .client
        .resolve_dispute(&escrow_id, &Symbol::new(&env, "refund_buyer"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_resolve_dispute_invalid_outcome() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let amount = 1000;

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    // Invalid outcome
    setup
        .client
        .resolve_dispute(&escrow_id, &Symbol::new(&env, "invalid_outcome"));
}

#[test]
fn test_circuit_breaker() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let _amount = 1000;
    setup.token_client.mint(&setup.buyer, &10000);

    let admin = Address::generate(&env);

    // Initialize
    setup.client.initialize(&admin);

    // Initialize again should panic
    // Wait, testing panic is easier in a separate test.

    // Pause contract
    setup.client.pause();

    // Try to create escrow - should fail
    // We can't catch the panic here easily in the same test if we want to continue, so we will just test it separately.
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_create_escrow_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let admin = Address::generate(&env);

    setup.client.initialize(&admin);
    setup.client.pause();

    setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.token_admin,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_pause_unauthorized() {
    let env = Env::default();
    let setup = setup_test(&env);
    let admin = Address::generate(&env);

    // We can't initialize using client without mock_all_auths if it requires auth? No, initialize doesn't require auth!
    setup.client.initialize(&admin);
    setup.client.pause();
}

#[test]
fn test_update_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    setup.client.initialize(&admin);

    // Update admin
    setup.client.update_admin(&new_admin);

    env.as_contract(&setup.contract_id, || {
        let current_admin = soroban_escrow_contracts::storage::read_admin(&env).unwrap();
        assert_eq!(current_admin, new_admin);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_admin_unauthorized() {
    let env = Env::default();
    let setup = setup_test(&env);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    setup.client.initialize(&admin);

    // Without auth, this should fail
    setup.client.update_admin(&new_admin);
}

#[test]
fn test_create_escrow_valid_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);

    let amount = 1000;
    let timeout = Some(150);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &timeout,
    );

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::storage::read_escrow_state(&env, escrow_id).unwrap();
        assert_eq!(state.timeout_ledger, timeout);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_create_escrow_timeout_equal_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);
    let amount = 1000;
    let timeout = Some(100);

    setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &timeout,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_create_escrow_timeout_past_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);
    let amount = 1000;
    let timeout = Some(50);

    setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &timeout,
    );
}

#[test]
fn test_refund_expired_success() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);

    let amount = 1000;
    let timeout = Some(150);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &timeout,
    );

    setup.client.lock_funds(&escrow_id);

    // Fast forward ledger past timeout
    env.ledger().set_sequence_number(151);

    setup.client.refund_expired(&escrow_id);

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
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_refund_expired_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);

    let amount = 1000;
    let timeout = Some(150);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &amount,
        &0,
        &setup.token_admin,
        &timeout,
    );

    setup.client.lock_funds(&escrow_id);

    // Ledger sequence not past timeout
    env.ledger().set_sequence_number(150);

    setup.client.refund_expired(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_refund_expired_unauthorized() {
    let env = Env::default();
    let setup = setup_test(&env);

    env.ledger().set_sequence_number(100);

    let timeout = Some(150);

    // Mint tokens manually using as_contract for unauthorized setup, or we can just create EscrowState directly
    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: setup.buyer.clone(),
            seller: setup.seller.clone(),
            token: setup.token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
            deadline: 0,
            mediator: setup.token_admin.clone(),
            timeout_ledger: timeout,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, 0, &state);
    });

    // Fast forward ledger past timeout
    env.ledger().set_sequence_number(151);

    // Call without auth
    setup.client.refund_expired(&0);
}
