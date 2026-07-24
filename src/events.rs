#![allow(deprecated)]
use crate::types::{EscrowId, EscrowState};
use soroban_sdk::{Address, Env, Symbol};

pub fn publish_escrow_created(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (Symbol::new(env, "EscrowCreated"), escrow_id);
    env.events().publish(topics, state.clone());
}

pub fn publish_funds_locked(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (Symbol::new(env, "FundsLocked"), escrow_id);
    env.events().publish(topics, state.clone());
}

pub fn publish_funds_released(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (Symbol::new(env, "FundsReleased"), escrow_id);
    env.events().publish(topics, state.clone());
}

pub fn publish_escrow_refunded(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (Symbol::new(env, "EscrowRefunded"), escrow_id);
    env.events().publish(topics, state.clone());
}

pub fn publish_dispute_resolved(
    env: &Env,
    escrow_id: EscrowId,
    state: &EscrowState,
    mediator: &Address,
    outcome: &Symbol,
) {
    let topics = (Symbol::new(env, "EscrowDisputeResolved"), escrow_id);
    env.events()
        .publish(topics, (state.clone(), mediator.clone(), outcome.clone()));
}

pub fn publish_protocol_fee_collected(
    env: &Env,
    escrow_id: EscrowId,
    treasury: &Address,
    fee: i128,
) {
    let topics = (Symbol::new(env, "ProtocolFeeCollected"), escrow_id);
    env.events().publish(topics, (treasury.clone(), fee));
}
