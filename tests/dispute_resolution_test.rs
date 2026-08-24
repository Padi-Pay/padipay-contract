#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

pub struct TestSetup<'a> {
    pub contract_id: Address,
    pub client: PadiPayEscrowContractClient<'a>,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub token_admin: Address,
    pub token_client: soroban_sdk::token::StellarAssetClient<'a>,
    pub token_client_basic: soroban_sdk::token::Client<'a>,
    pub mediator: Address,
}

pub fn setup_test<'a>(env: &'a Env) -> TestSetup<'a> {
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);

    let buyer = Address::generate(env);
    let seller = Address::generate(env);
    let mediator = Address::generate(env);

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
        mediator,
    }
}

#[test]
fn test_resolve_dispute_refund_buyer() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.mediator,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);

    let outcome = Symbol::new(&env, "refund_buyer");
    setup.client.resolve_dispute(&escrow_id, &outcome);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

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

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.mediator,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);
    assert_eq!(setup.token_client_basic.balance(&setup.seller), 0);

    let outcome = Symbol::new(&env, "pay_seller");
    setup.client.resolve_dispute(&escrow_id, &outcome);

    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);
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
fn test_resolve_dispute_replay_attack_refunded() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.mediator,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    let outcome = Symbol::new(&env, "refund_buyer");
    setup.client.resolve_dispute(&escrow_id, &outcome);

    // Replay attack
    setup.client.resolve_dispute(&escrow_id, &outcome);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_resolve_dispute_replay_attack_released() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.mediator,
        &None,
    );

    setup.client.lock_funds(&escrow_id);

    let outcome = Symbol::new(&env, "pay_seller");
    setup.client.resolve_dispute(&escrow_id, &outcome);

    // Replay attack
    setup.client.resolve_dispute(&escrow_id, &outcome);
}
