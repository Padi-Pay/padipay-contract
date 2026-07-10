use crate::error::Error;
use crate::storage::{read_escrow_state, write_escrow_state};
use crate::types::{EscrowState, EscrowStatus};
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
        Ok(())
    }
    /// Locks funds in the escrow.
    pub fn lock_funds(env: Env) -> Result<(), Error> {
        let mut state = read_escrow_state(&env)?;

        state.buyer.require_auth();

        if state.status != EscrowStatus::Created {
            return Err(Error::EscrowAlreadyFunded);
        }

        if !state.status.is_valid_transition(&EscrowStatus::Locked) {
            return Err(Error::InvalidState);
        }

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from buyer to contract
        token_client.transfer(&state.buyer, &env.current_contract_address(), &state.amount);

        state.status = EscrowStatus::Locked;
        write_escrow_state(&env, &state);

        Ok(())
    }

    /// Releases funds to the seller.
    pub fn release_funds(env: Env) -> Result<(), Error> {
        let mut state = read_escrow_state(&env)?;

        state.buyer.require_auth();

        if !state.status.is_valid_transition(&EscrowStatus::Released) {
            return Err(Error::InvalidState);
        }

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract to seller
        token_client.transfer(
            &env.current_contract_address(),
            &state.seller,
            &state.amount,
        );

        state.status = EscrowStatus::Released;
        write_escrow_state(&env, &state);

        Ok(())
    }

    /// Refunds funds back to the buyer.
    pub fn refund(env: Env) -> Result<(), Error> {
        let mut state = read_escrow_state(&env)?;

        state.seller.require_auth();

        if !state.status.is_valid_transition(&EscrowStatus::Refunded) {
            return Err(Error::InvalidState);
        }

        let token_client = crate::token::get_token_client(&env, &state.token);

        // Transfer from contract back to buyer
        token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);

        state.status = EscrowStatus::Refunded;
        write_escrow_state(&env, &state);

        Ok(())
    }

    /// Resolves a dispute between buyer and seller.
    pub fn resolve_dispute(env: Env, mediator: Address, outcome: Symbol) {
        // TODO: Verify the mediator has authorized the action and is an approved admin.
        // TODO: Retrieve the escrow state. Ensure it is not already 'Released'.
        // TODO: Parse the `outcome` (e.g., "refund_buyer" or "pay_seller").
        // TODO: Transfer funds accordingly and update the state to terminal.
        // TODO: Emit an event detailing the dispute resolution.
    }
}
