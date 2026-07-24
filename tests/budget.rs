//! Soroban compute budget assertions for the PadiPay escrow contract.
//!
//! Soroban transactions revert on Mainnet if they exceed the network's CPU
//! instruction or memory budget. If the contract unintentionally starts
//! doing more work per call (an accidental loop, an inefficient rewrite of
//! a hot path, a new dependency that inflates Wasm cost), unit tests can
//! still pass while the contract silently becomes unusable in production.
//!
//! These tests assert that every state-changing entry point stays under a
//! conservative ceiling, and print the metered cost of each call so
//! resource usage is visible to anyone reviewing a PR.
#![cfg(test)]

use soroban_escrow_contracts::{PadiPayEscrowContract, PadiPayEscrowContractClient};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Bytes, Env, Symbol};

pub struct BudgetTestSetup<'a> {
    pub client: PadiPayEscrowContractClient<'a>,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub token_client: soroban_sdk::token::StellarAssetClient<'a>,
}

pub fn setup<'a>(env: &'a Env) -> BudgetTestSetup<'a> {
    let contract_id = env.register(PadiPayEscrowContract, ());
    let client = PadiPayEscrowContractClient::new(env, &contract_id);

    let buyer = Address::generate(env);
    let seller = Address::generate(env);

    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token = token_contract.address();
    let token_client = soroban_sdk::token::StellarAssetClient::new(env, &token);

    BudgetTestSetup {
        client,
        buyer,
        seller,
        token,
        token_client,
    }
}

/// Prints the metered cost of the last top-level invocation - via
/// `env.budget().print()` equivalent (`cost_estimate().budget()` is the
/// non-deprecated form in SDK 26) - so it's visible in CI logs, then
/// asserts it stayed within the given ceilings.
///
/// The budget resets automatically before every top-level contract
/// invocation, so this must be called immediately after the call it is
/// measuring.
fn assert_within_budget(env: &Env, label: &str, max_cpu_instructions: u64, max_memory_bytes: u64) {
    let budget = env.cost_estimate().budget();
    budget.print();

    let cpu = budget.cpu_instruction_cost();
    let mem = budget.memory_bytes_cost();

    assert!(
        cpu <= max_cpu_instructions,
        "{label}: CPU instructions {cpu} exceeded budget ceiling {max_cpu_instructions}"
    );
    assert!(
        mem <= max_memory_bytes,
        "{label}: memory bytes {mem} exceeded budget ceiling {max_memory_bytes}"
    );
}

// Conservative ceilings for the contract's entry points, calibrated with
// roughly 2-2.5x headroom over measured usage on soroban-sdk 26. This is
// far below the Stellar Mainnet per-invocation limit (600M instructions /
// ~42MB memory) by design: the goal is catching a regression in CI, not
// merely avoiding an on-chain revert.
const CREATE_ESCROW_MAX_CPU: u64 = 250_000;
const CREATE_ESCROW_MAX_MEM: u64 = 100_000;

// lock_funds, release_funds, refund, and resolve_dispute all perform a
// single token transfer plus escrow state read/write, so they share a
// ceiling calibrated off that shape of workload.
const TRANSFER_OP_MAX_CPU: u64 = 700_000;
const TRANSFER_OP_MAX_MEM: u64 = 250_000;

#[test]
fn test_create_escrow_within_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup(&env);

    setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &1000);

    assert_within_budget(
        &env,
        "create_escrow",
        CREATE_ESCROW_MAX_CPU,
        CREATE_ESCROW_MAX_MEM,
    );
}

#[test]
fn test_lock_funds_within_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup(&env);

    setup.token_client.mint(&setup.buyer, &10_000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &1000);

    setup.client.lock_funds(&escrow_id);

    assert_within_budget(&env, "lock_funds", TRANSFER_OP_MAX_CPU, TRANSFER_OP_MAX_MEM);
}

#[test]
fn test_release_funds_within_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup(&env);

    setup.token_client.mint(&setup.buyer, &10_000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &1000);
    setup.client.lock_funds(&escrow_id);

    setup.client.release_funds(&escrow_id);

    assert_within_budget(
        &env,
        "release_funds",
        TRANSFER_OP_MAX_CPU,
        TRANSFER_OP_MAX_MEM,
    );
}

#[test]
fn test_refund_within_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup(&env);

    setup.token_client.mint(&setup.buyer, &10_000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &1000);
    setup.client.lock_funds(&escrow_id);

    setup.client.refund(&escrow_id);

    assert_within_budget(&env, "refund", TRANSFER_OP_MAX_CPU, TRANSFER_OP_MAX_MEM);
}

#[test]
fn test_resolve_dispute_within_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let setup = setup(&env);
    let mediator = Address::generate(&env);

    setup.token_client.mint(&setup.buyer, &10_000);
    let escrow_id = setup
        .client
        .create_escrow(&setup.buyer, &setup.seller, &setup.token, &1000);
    setup.client.lock_funds(&escrow_id);

    setup
        .client
        .resolve_dispute(&escrow_id, &mediator, &Symbol::new(&env, "pay_seller"));

    assert_within_budget(
        &env,
        "resolve_dispute",
        TRANSFER_OP_MAX_CPU,
        TRANSFER_OP_MAX_MEM,
    );
}

// ---------------------------------------------------------------------
// Proof that the budget assertion actually catches a violation.
//
// This is a standalone dummy contract with a deliberately heavy loop -
// it is not part of the audited escrow contract and exists only to
// demonstrate that `assert_within_budget` fails the build when resource
// usage regresses past the ceiling, rather than trivially passing.
// ---------------------------------------------------------------------

#[contract]
struct BudgetStressContract;

#[contractimpl]
impl BudgetStressContract {
    pub fn heavy_loop(env: Env, iterations: u32) {
        for i in 0..iterations {
            let payload = Bytes::from_array(&env, &i.to_be_bytes());
            env.crypto().sha256(&payload);
        }
    }
}

#[test]
#[should_panic(expected = "exceeded budget ceiling")]
fn test_budget_assertion_catches_heavy_loop() {
    let env = Env::default();
    let contract_id = env.register(BudgetStressContract, ());
    let client = BudgetStressContractClient::new(&env, &contract_id);

    // Same order of magnitude as the real entry points' ceilings, but the
    // workload below is intentionally heavy enough to blow past it.
    client.heavy_loop(&5_000);

    assert_within_budget(&env, "heavy_loop", TRANSFER_OP_MAX_CPU, TRANSFER_OP_MAX_MEM);
}
