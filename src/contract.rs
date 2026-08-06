use crate::error::Error;
use crate::events::{
    publish_escrow_created, publish_escrow_refunded, publish_funds_locked, publish_funds_released,
};
use crate::storage::{increment_nonce, write_escrow_state};
use crate::types::{EscrowId, EscrowState, EscrowStatus};
use crate::validation::{
    require_buyer, require_escrow, require_seller, require_status, require_valid_transition,
};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct PadiPayEscrowContract;

#[contractimpl]
impl PadiPayEscrowContract {
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
        deadline: u64,
        mediator: Address,
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
            deadline,
            mediator,
        };
        let id = increment_nonce(&env);
        write_escrow_state(&env, id, &state);
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

        publish_escrow_refunded(&env, escrow_id, &state);

        Ok(())
    }

    /// Executes a timeout if the deadline has passed, refunding the buyer.
    pub fn execute_timeout(env: Env, escrow_id: EscrowId) -> Result<(), Error> {
        let mut state = require_escrow(&env, escrow_id)?;

        let current_time = env.ledger().timestamp();
        if current_time <= state.deadline {
            return Err(Error::DeadlineNotReached);
        }

        require_valid_transition(&state, &EscrowStatus::Refunded)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract back to buyer
        token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);

        state.status = EscrowStatus::Refunded;
        write_escrow_state(&env, escrow_id, &state);

        publish_escrow_refunded(&env, escrow_id, &state);

        Ok(())
    }

    /// Resolves a dispute between buyer and seller.
    pub fn resolve_dispute(env: Env, escrow_id: EscrowId, outcome: Symbol) -> Result<(), Error> {
        let mut state = require_escrow(&env, escrow_id)?;

        state.mediator.require_auth();

        let token_client = crate::token::get_token_client(&env, &state.token);

        if outcome == Symbol::new(&env, "refund_buyer") {
            require_valid_transition(&state, &EscrowStatus::Refunded)?;
            token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);
            state.status = EscrowStatus::Refunded;
            publish_escrow_refunded(&env, escrow_id, &state);
        } else if outcome == Symbol::new(&env, "pay_seller") {
            require_valid_transition(&state, &EscrowStatus::Released)?;
            token_client.transfer(&env.current_contract_address(), &state.seller, &state.amount);
            state.status = EscrowStatus::Released;
            publish_funds_released(&env, escrow_id, &state);
        } else {
            return Err(Error::InvalidState); // Unknown outcome
        }

        write_escrow_state(&env, escrow_id, &state);
        Ok(())
    }
}
