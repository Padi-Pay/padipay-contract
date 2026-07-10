use crate::error::Error;
use crate::storage::read_escrow_state;
use crate::types::{EscrowState, EscrowStatus};
use soroban_sdk::Env;

/// Validates escrow existence and retrieves the state.
pub fn require_escrow(env: &Env) -> Result<EscrowState, Error> {
    read_escrow_state(env)
}

/// Validates escrow ownership by the buyer.
pub fn require_buyer(state: &EscrowState) {
    state.buyer.require_auth();
}

/// Validates escrow ownership by the seller.
pub fn require_seller(state: &EscrowState) {
    state.seller.require_auth();
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
