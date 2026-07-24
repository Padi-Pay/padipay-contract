use crate::error::Error;
use crate::types::{DataKey, EscrowId, EscrowState};
use soroban_sdk::{Address, Env};

// ~30 days in ledgers (assuming 5s per ledger)
const DAY_IN_LEDGERS: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = 14 * DAY_IN_LEDGERS;
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 14 * DAY_IN_LEDGERS;

/// Extends the TTL of the instance storage.
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Extends the TTL of a persistent storage entry.
pub fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

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

/// Writes the protocol fee rate and treasury address to instance storage.
pub fn write_fee_config(env: &Env, rate: u32, treasury: &Address) {
    env.storage().instance().set(&DataKey::FeeRate, &rate);
    env.storage().instance().set(&DataKey::Treasury, treasury);
}

/// Reads the protocol fee rate from instance storage.
pub fn read_fee_rate(env: &Env) -> Option<u32> {
    env.storage().instance().get(&DataKey::FeeRate)
}

/// Reads the treasury address from instance storage.
pub fn read_treasury(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Treasury)
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
