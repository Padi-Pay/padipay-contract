use crate::error::EscrowError;
use crate::types::{DataKey, EscrowState};
use soroban_sdk::Env;

/// Reads the escrow state from storage.
pub fn read_escrow_state(env: &Env) -> Result<EscrowState, EscrowError> {
    env.storage()
        .instance()
        .get(&DataKey::State)
        .ok_or(EscrowError::NotFound)
}

/// Writes the escrow state to storage.
pub fn write_escrow_state(env: &Env, state: &EscrowState) {
    env.storage().instance().set(&DataKey::State, state);
}

/// Updates the escrow state in storage, ensuring it already exists.
pub fn update_escrow_state(env: &Env, state: &EscrowState) -> Result<(), EscrowError> {
    if !env.storage().instance().has(&DataKey::State) {
        return Err(EscrowError::NotFound);
    }
    write_escrow_state(env, state);
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
            assert_eq!(read_escrow_state(&env), Err(EscrowError::NotFound));
            assert_eq!(
                update_escrow_state(&env, &state),
                Err(EscrowError::NotFound)
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
}
