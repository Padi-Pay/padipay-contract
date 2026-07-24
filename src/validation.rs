use crate::error::Error;
use crate::storage::read_escrow_state;
use crate::types::{EscrowId, EscrowState, EscrowStatus};
use soroban_sdk::{Address, Env};

/// Validates escrow existence and retrieves the state.
pub fn require_escrow(env: &Env, id: EscrowId) -> Result<EscrowState, Error> {
    read_escrow_state(env, id)
}

/// Validates that the buyer has authorized the escrow creation.
pub fn require_buyer_auth(buyer: &Address) {
    buyer.require_auth();
}

/// Validates escrow ownership by the buyer.
pub fn require_buyer(state: &EscrowState) {
    state.buyer.require_auth();
}

/// Validates escrow ownership by the seller.
pub fn require_seller(state: &EscrowState) {
    state.seller.require_auth();
}

/// Validates that the administrator/mediator has authorized the transaction.
pub fn require_admin(admin: &Address) {
    admin.require_auth();
}

/// Validates the current escrow status exactly matches the expected status.
pub fn require_status(state: &EscrowState, expected: &EscrowStatus) -> Result<(), Error> {
    if &state.status != expected {
        if *expected == EscrowStatus::Created && state.status != EscrowStatus::Created {
            return Err(Error::EscrowAlreadyFunded);
        }
        return Err(Error::InvalidState);
    }
    Ok(())
}

/// Validates that the escrow can transition to a new status.
pub fn require_valid_transition(state: &EscrowState, target: &EscrowStatus) -> Result<(), Error> {
    if !state.status.is_valid_transition(target) {
        return Err(Error::InvalidState);
    }
    Ok(())
}

/// Validates that the escrow has not already reached a terminal state.
pub fn require_not_terminal(state: &EscrowState) -> Result<(), Error> {
    if matches!(
        state.status,
        EscrowStatus::Released | EscrowStatus::Refunded
    ) {
        return Err(Error::InvalidState);
    }
    Ok(())
}

/// The maximum protocol fee rate, in basis points (10%).
pub const MAX_FEE_RATE_BPS: u32 = 1000;

/// Validates that a fee rate does not exceed the protocol maximum.
pub fn require_valid_fee_rate(rate: u32) -> Result<(), Error> {
    if rate > MAX_FEE_RATE_BPS {
        return Err(Error::InvalidFeeRate);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_require_seller_success() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(crate::PadiPayEscrowContract, ());

        env.as_contract(&contract_id, || {
            let buyer = Address::generate(&env);
            let seller = Address::generate(&env);
            let token = Address::generate(&env);

            let state = EscrowState {
                buyer,
                seller,
                token,
                amount: 100,
                status: EscrowStatus::Created,
            };

            // This will succeed because mock_all_auths() is called
            require_seller(&state);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_require_seller_unauthorized() {
        let env = Env::default();
        let contract_id = env.register(crate::PadiPayEscrowContract, ());

        env.as_contract(&contract_id, || {
            let buyer = Address::generate(&env);
            let seller = Address::generate(&env);
            let token = Address::generate(&env);

            let state = EscrowState {
                buyer,
                seller,
                token,
                amount: 100,
                status: EscrowStatus::Created,
            };

            // This will panic because no auths are mocked
            require_seller(&state);
        });
    }
}
