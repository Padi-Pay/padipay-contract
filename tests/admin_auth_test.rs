#![cfg(test)]

use soroban_escrow_contracts::{error::Error, PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, Symbol,
};

pub struct TestSetup<'a> {
    pub contract_id: Address,
    pub client: PadiPayEscrowContractClient<'a>,
    pub admin: Address,
}

pub fn setup_test<'a>(env: &'a Env) -> TestSetup<'a> {
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);

    TestSetup {
        contract_id,
        client,
        admin,
    }
}

#[test]
fn test_reinitialization_reverts() {
    let env = Env::default();
    let setup = setup_test(&env);
    let new_admin = Address::generate(&env);

    let res = setup.client.try_initialize(&new_admin);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_admin_missing_auth() {
    let env = Env::default();
    let setup = setup_test(&env);
    let new_admin = Address::generate(&env);

    // Missing auth will cause require_auth() to panic with Auth error
    env.mock_auths(&[]);
    setup.client.update_admin(&new_admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_admin_spoofed_auth() {
    let env = Env::default();
    let setup = setup_test(&env);
    let new_admin = Address::generate(&env);
    let spoofed_admin = Address::generate(&env);

    // Spoofed auth: someone else signs it
    env.mock_auths(&[MockAuth {
        address: &spoofed_admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "update_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.update_admin(&new_admin);
}

#[test]
fn test_update_admin_correct_auth() {
    let env = Env::default();
    let setup = setup_test(&env);
    let new_admin = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &setup.admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "update_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.update_admin(&new_admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_pause_missing_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    env.mock_auths(&[]);
    setup.client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_pause_spoofed_auth() {
    let env = Env::default();
    let setup = setup_test(&env);
    let spoofed_admin = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &spoofed_admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "pause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.pause();
}

#[test]
fn test_pause_correct_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    env.mock_auths(&[MockAuth {
        address: &setup.admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "pause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.pause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unpause_missing_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    env.mock_auths(&[]);
    setup.client.unpause();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unpause_spoofed_auth() {
    let env = Env::default();
    let setup = setup_test(&env);
    let spoofed_admin = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &spoofed_admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "unpause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.unpause();
}

#[test]
fn test_unpause_correct_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    env.mock_auths(&[MockAuth {
        address: &setup.admin,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "unpause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.unpause();
}

// For resolve_dispute, the admin boundary is the `mediator`.
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_resolve_dispute_missing_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    // Mock an escrow state with a specific mediator
    let mediator = Address::generate(&env);
    let escrow_id = 1u64;

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: Address::generate(&env),
            seller: Address::generate(&env),
            token: Address::generate(&env),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
            deadline: 0,
            mediator: mediator.clone(),
            timeout_ledger: None,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, escrow_id, &state);
    });

    env.mock_auths(&[]);
    setup
        .client
        .resolve_dispute(&escrow_id, &Symbol::new(&env, "refund_buyer"));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_resolve_dispute_spoofed_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    let mediator = Address::generate(&env);
    let spoofed_mediator = Address::generate(&env);
    let escrow_id = 1u64;
    let outcome = Symbol::new(&env, "refund_buyer");

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: Address::generate(&env),
            seller: Address::generate(&env),
            token: Address::generate(&env),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
            deadline: 0,
            mediator: mediator.clone(),
            timeout_ledger: None,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, escrow_id, &state);
    });

    env.mock_auths(&[MockAuth {
        address: &spoofed_mediator,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "resolve_dispute",
            args: (escrow_id, outcome.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.resolve_dispute(&escrow_id, &outcome);
}

#[test]
fn test_resolve_dispute_correct_auth() {
    let env = Env::default();
    let setup = setup_test(&env);

    let buyer = Address::generate(&env);
    let token = Address::generate(&env);
    let mediator = Address::generate(&env);
    let escrow_id = 1u64;
    let outcome = Symbol::new(&env, "refund_buyer");

    // Setup a real token contract so transfer works
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token = token_contract.address();
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token);
    env.mock_all_auths();
    token_client.mint(&setup.contract_id, &1000);

    env.as_contract(&setup.contract_id, || {
        let state = soroban_escrow_contracts::types::EscrowState {
            buyer: buyer.clone(),
            seller: Address::generate(&env),
            token: token.clone(),
            amount: 1000,
            status: soroban_escrow_contracts::types::EscrowStatus::Locked,
            deadline: 0,
            mediator: mediator.clone(),
            timeout_ledger: None,
        };
        soroban_escrow_contracts::storage::write_escrow_state(&env, escrow_id, &state);
    });

    env.mock_auths(&[MockAuth {
        address: &mediator,
        invoke: &MockAuthInvoke {
            contract: &setup.contract_id,
            fn_name: "resolve_dispute",
            args: (escrow_id, outcome.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    setup.client.resolve_dispute(&escrow_id, &outcome);
}
