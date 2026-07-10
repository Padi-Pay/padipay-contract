use crate::error::Error;
use crate::types::{DataKey, EscrowState};
use soroban_sdk::Env;

/// Reads the escrow state from storage.
pub fn read_escrow_state(env: &Env) -> Result<EscrowState, Error> {
    env.storage()
        .instance()
        .get(&DataKey::State)
        .ok_or(Error::EscrowNotFound)
}

/// Writes the escrow state to storage.
pub fn write_escrow_state(env: &Env, state: &EscrowState) {
    env.storage().instance().set(&DataKey::State, state);
}

/// Updates the escrow state in storage, ensuring it already exists.
pub fn update_escrow_state(env: &Env, state: &EscrowState) -> Result<(), Error> {
    if !env.storage().instance().has(&DataKey::State) {
        return Err(Error::EscrowNotFound);
    }
    write_escrow_state(env, state);
    Ok(())
}

/// Verifies that the seller has authorized the current invocation.
pub fn verify_seller(env: &Env) -> Result<(), Error> {
    let state = read_escrow_state(env)?;
    state.seller.require_auth();
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::EscrowStatus;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_storage_helpers() {
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

            // Initially not found
            assert_eq!(read_escrow_state(&env), Err(Error::EscrowNotFound));
            assert_eq!(
                update_escrow_state(&env, &state),
                Err(Error::EscrowNotFound)
            );

            // Write and read
            write_escrow_state(&env, &state);
            assert_eq!(read_escrow_state(&env), Ok(state.clone()));

            // Update
            let mut new_state = state.clone();
            new_state.status = EscrowStatus::Locked;
            assert_eq!(update_escrow_state(&env, &new_state), Ok(()));
            assert_eq!(read_escrow_state(&env), Ok(new_state));
        });
    }

    #[test]
    fn test_verify_seller_success() {
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
            write_escrow_state(&env, &state);

            assert_eq!(verify_seller(&env), Ok(()));
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn test_verify_seller_unauthorized() {
        let env = Env::default();
        // Do NOT mock auths
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
            write_escrow_state(&env, &state);

            // This should panic because seller didn't authorize
            let _ = verify_seller(&env);
        });
    }
}
