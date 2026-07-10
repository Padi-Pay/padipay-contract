#![allow(deprecated)]
use crate::types::EscrowState;
use soroban_sdk::{Env, Symbol};

pub fn publish_escrow_created(env: &Env, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "EscrowCreated"),
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_funds_locked(env: &Env, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "FundsLocked"),
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_funds_released(env: &Env, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "FundsReleased"),
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}

pub fn publish_escrow_refunded(env: &Env, state: &EscrowState) {
    let topics = (
        Symbol::new(env, "EscrowRefunded"),
        state.buyer.clone(),
        state.seller.clone(),
    );
    env.events().publish(topics, state.amount);
}
