use crate::error::Error;
use crate::types::{DataKey, EscrowId, EscrowState};
use soroban_sdk::Env;

/// Reads the escrow state from storage.
pub fn read_escrow_state(env: &Env, id: EscrowId) -> Result<EscrowState, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(id))
        .ok_or(Error::EscrowNotFound)
}

/// Writes the escrow state to storage.
pub fn write_escrow_state(env: &Env, id: EscrowId, state: &EscrowState) {
    env.storage().persistent().set(&DataKey::Escrow(id), state);
}

/// Updates the escrow state in storage, ensuring it already exists.
pub fn update_escrow_state(env: &Env, id: EscrowId, state: &EscrowState) -> Result<(), Error> {
    if !env.storage().persistent().has(&DataKey::Escrow(id)) {
        return Err(Error::EscrowNotFound);
    }
    write_escrow_state(env, id, state);
    Ok(())
}

/// Generates a monotonically increasing, unique Escrow ID.
pub fn increment_nonce(env: &Env) -> EscrowId {
    let mut nonce: EscrowId = env
        .storage()
        .instance()
        .get(&DataKey::EscrowNonce)
        .unwrap_or(0);

    nonce += 1;

    env.storage().instance().set(&DataKey::EscrowNonce, &nonce);

    nonce
}

/// Reads the admin address.
pub fn read_admin(env: &Env) -> Result<soroban_sdk::Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

/// Writes the admin address.
pub fn write_admin(env: &Env, admin: &soroban_sdk::Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Checks if the contract is paused.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

/// Sets the paused state of the contract.
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::IsPaused, &paused);
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
                deadline: 0,
                mediator: soroban_sdk::Address::generate(&env),
                timeout_ledger: None,
            };

            let id: EscrowId = 1;

            // Initially not found
            assert_eq!(read_escrow_state(&env, id), Err(Error::EscrowNotFound));
            assert_eq!(
                update_escrow_state(&env, id, &state),
                Err(Error::EscrowNotFound)
            );

            // Write and read
            write_escrow_state(&env, id, &state);
            assert_eq!(read_escrow_state(&env, id), Ok(state.clone()));

            // Update
            let mut new_state = state.clone();
            new_state.status = EscrowStatus::Locked;
            assert_eq!(update_escrow_state(&env, id, &new_state), Ok(()));
            assert_eq!(read_escrow_state(&env, id), Ok(new_state));
        });
    }

    #[test]
    fn test_increment_nonce() {
        let env = Env::default();
        let contract_id = env.register(crate::PadiPayEscrowContract, ());

        env.as_contract(&contract_id, || {
            assert_eq!(increment_nonce(&env), 1);
            assert_eq!(increment_nonce(&env), 2);
            assert_eq!(increment_nonce(&env), 3);
        });
    }
}
