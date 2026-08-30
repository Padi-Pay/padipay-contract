#![allow(deprecated)]
use crate::types::{EscrowId, EscrowState};
use soroban_sdk::{Env, Symbol};

/// Schema version for every event emitted by this contract.
///
/// This symbol is published as the second topic of every event, directly after
/// the event name. Off-chain indexers can filter on it to ignore events whose
/// payload shape they do not yet understand. Bump this identifier (`v2`, `v3`,
/// ...) whenever the topic tuple or the data payload of any event changes in a
/// backward-incompatible way. See `docs/integration.md` for the full strategy.
pub const EVENT_SCHEMA_VERSION: &str = "v1";

fn version(env: &Env) -> Symbol {
    Symbol::new(env, EVENT_SCHEMA_VERSION)
}

pub fn publish_escrow_created(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "EscrowCreated"),
        version(env),
        escrow_id,
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_funds_locked(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "FundsLocked"),
        version(env),
        escrow_id,
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_funds_released(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "FundsReleased"),
        version(env),
        escrow_id,
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_escrow_refunded(env: &Env, escrow_id: EscrowId, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "EscrowRefunded"),
        version(env),
        escrow_id,
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}
