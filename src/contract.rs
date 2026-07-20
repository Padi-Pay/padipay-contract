use crate::error::Error;
use crate::events::{
    publish_escrow_created, publish_escrow_refunded, publish_funds_locked, publish_funds_released,
};
use crate::storage::{
    extend_instance_ttl, extend_persistent_ttl, increment_nonce, write_escrow_state,
};
use crate::types::{DataKey, EscrowId, EscrowState, EscrowStatus};
use crate::validation::{
    require_buyer, require_escrow, require_seller, require_status, require_valid_transition,
};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct PadiPayEscrowContract;

#[contractimpl]
impl PadiPayEscrowContract {
    /// Creates a new escrow agreement.
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
    ) -> Result<EscrowId, Error> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if buyer == seller {
            return Err(Error::InvalidAddresses);
        }

        let state = EscrowState {
            buyer,
            seller,
            token,
            amount,
            status: EscrowStatus::Created,
        };
        let id = increment_nonce(&env);
        write_escrow_state(&env, id, &state);

        extend_instance_ttl(&env);
        extend_persistent_ttl(&env, &DataKey::Escrow(id));

        publish_escrow_created(&env, id, &state);
        Ok(id)
    }
    /// Locks funds in the escrow.
    pub fn lock_funds(env: Env, escrow_id: EscrowId) -> Result<(), Error> {
        let mut state = require_escrow(&env, escrow_id)?;

        require_buyer(&state);
        require_status(&state, &EscrowStatus::Created)?;
        require_valid_transition(&state, &EscrowStatus::Locked)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from buyer to contract
        token_client.transfer(&state.buyer, env.current_contract_address(), &state.amount);

        state.status = EscrowStatus::Locked;
        write_escrow_state(&env, escrow_id, &state);

        extend_instance_ttl(&env);
        extend_persistent_ttl(&env, &DataKey::Escrow(escrow_id));

        publish_funds_locked(&env, escrow_id, &state);

        Ok(())
    }

    /// Releases funds to the seller.
    pub fn release_funds(env: Env, escrow_id: EscrowId) -> Result<(), Error> {
        let mut state = require_escrow(&env, escrow_id)?;

        require_buyer(&state);
        require_valid_transition(&state, &EscrowStatus::Released)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract to seller
        token_client.transfer(
            &env.current_contract_address(),
            &state.seller,
            &state.amount,
        );

        state.status = EscrowStatus::Released;
        write_escrow_state(&env, escrow_id, &state);

        extend_instance_ttl(&env);
        extend_persistent_ttl(&env, &DataKey::Escrow(escrow_id));

        publish_funds_released(&env, escrow_id, &state);

        Ok(())
    }

    /// Refunds funds back to the buyer.
    pub fn refund(env: Env, escrow_id: EscrowId) -> Result<(), Error> {
        let mut state = require_escrow(&env, escrow_id)?;

        require_seller(&state);
        require_valid_transition(&state, &EscrowStatus::Refunded)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract back to buyer
        token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);

        state.status = EscrowStatus::Refunded;
        write_escrow_state(&env, escrow_id, &state);

        extend_instance_ttl(&env);
        extend_persistent_ttl(&env, &DataKey::Escrow(escrow_id));

        publish_escrow_refunded(&env, escrow_id, &state);

        Ok(())
    }

    /// Resolves a dispute between buyer and seller.
    pub fn resolve_dispute(env: Env, escrow_id: EscrowId, _mediator: Address, outcome: Symbol) {
        // TODO: Verify the mediator has authorized the action and is an approved admin.

        let mut state = require_escrow(&env, escrow_id).unwrap();

        let token_client = crate::token::get_token_client(&env, &state.token);

        if outcome == Symbol::new(&env, "refund_buyer") {
            token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);
            state.status = EscrowStatus::Refunded;
        } else if outcome == Symbol::new(&env, "pay_seller") {
            token_client.transfer(
                &env.current_contract_address(),
                &state.seller,
                &state.amount,
            );
            state.status = EscrowStatus::Released;
        } else {
            panic!("Invalid outcome");
        }

        write_escrow_state(&env, escrow_id, &state);

        extend_instance_ttl(&env);
        extend_persistent_ttl(&env, &DataKey::Escrow(escrow_id));

        // TODO: Emit an event detailing the dispute resolution.
    }
}
