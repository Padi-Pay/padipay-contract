use crate::error::Error;
use crate::events::{
    publish_escrow_created, publish_escrow_refunded, publish_funds_locked, publish_funds_released,
};
use crate::storage::write_escrow_state;
use crate::types::{EscrowState, EscrowStatus};
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
    ) -> Result<(), Error> {
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
        write_escrow_state(&env, &state);
        publish_escrow_created(&env, &state);
        Ok(())
    }
    /// Locks funds in the escrow.
    pub fn lock_funds(env: Env) -> Result<(), Error> {
        let mut state = require_escrow(&env)?;

        require_buyer(&state);
        require_status(&state, &EscrowStatus::Created)?;
        require_valid_transition(&state, &EscrowStatus::Locked)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from buyer to contract
        token_client.transfer(&state.buyer, env.current_contract_address(), &state.amount);

        state.status = EscrowStatus::Locked;
        write_escrow_state(&env, &state);

        publish_funds_locked(&env, &state);

        Ok(())
    }

    /// Releases funds to the seller.
    pub fn release_funds(env: Env) -> Result<(), Error> {
        let mut state = require_escrow(&env)?;

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
        write_escrow_state(&env, &state);

        publish_funds_released(&env, &state);

        Ok(())
    }

    /// Refunds funds back to the buyer.
    pub fn refund(env: Env) -> Result<(), Error> {
        let mut state = require_escrow(&env)?;

        require_seller(&state);
        require_valid_transition(&state, &EscrowStatus::Refunded)?;

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract back to buyer
        token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);

        state.status = EscrowStatus::Refunded;
        write_escrow_state(&env, &state);

        publish_escrow_refunded(&env, &state);

        Ok(())
    }

    /// Resolves a dispute between buyer and seller.
    pub fn resolve_dispute(_env: Env, _mediator: Address, _outcome: Symbol) {
        // TODO: Verify the mediator has authorized the action and is an approved admin.
        // TODO: Retrieve the escrow state. Ensure it is not already 'Released'.
        // TODO: Parse the `outcome` (e.g., "refund_buyer" or "pay_seller").
        // TODO: Transfer funds accordingly and update the state to terminal.
        // TODO: Emit an event detailing the dispute resolution.
    }
}
