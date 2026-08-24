#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
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
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_refund_expired_sequence_less_than_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    let timeout = 150;
    env.ledger().set_sequence_number(100);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.token_admin,
        &Some(timeout),
    );

    setup.client.lock_funds(&escrow_id);

    // Sequence < timeout
    env.ledger().set_sequence_number(149);

    setup.client.refund_expired(&escrow_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_refund_expired_sequence_equal_to_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    let timeout = 150;
    env.ledger().set_sequence_number(100);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.token_admin,
        &Some(timeout),
    );

    setup.client.lock_funds(&escrow_id);

    // Sequence == timeout
    env.ledger().set_sequence_number(150);

    setup.client.refund_expired(&escrow_id);
}

#[test]
fn test_refund_expired_sequence_greater_than_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    let timeout = 150;
    env.ledger().set_sequence_number(100);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.token_admin,
        &Some(timeout),
    );

    setup.client.lock_funds(&escrow_id);

    // Balance before refund
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 1000);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 9000);

    // Sequence > timeout
    env.ledger().set_sequence_number(151);

    setup.client.refund_expired(&escrow_id);

    // Balance after refund
    assert_eq!(setup.token_client_basic.balance(&setup.contract_id), 0);
    assert_eq!(setup.token_client_basic.balance(&setup.buyer), 10000);

    // Escrow Status should be Refunded
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
fn test_refund_expired_replay_fails_on_already_refunded_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup_test(&env);

    let timeout = 150;
    env.ledger().set_sequence_number(100);

    setup.token_client.mint(&setup.buyer, &10000);

    let escrow_id = setup.client.create_escrow(
        &setup.buyer,
        &setup.seller,
        &setup.token,
        &1000,
        &0,
        &setup.token_admin,
        &Some(timeout),
    );

    setup.client.lock_funds(&escrow_id);

    // Sequence > timeout
    env.ledger().set_sequence_number(151);

    setup.client.refund_expired(&escrow_id);

    // Replay attempt should panic with InvalidState (Error 2)
    setup.client.refund_expired(&escrow_id);
}
