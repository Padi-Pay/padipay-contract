use soroban_sdk::{token, Address, Env};

/// Creates a token client for the specified token contract address.
pub fn get_token_client<'a>(env: &'a Env, token: &Address) -> token::Client<'a> {
    token::Client::new(env, token)
}

/// Safely queries the balance of a token holder.
///
/// # Arguments
/// * `env` - The current contract execution environment.
/// * `token` - The address of the token contract.
/// * `owner` - The address of the balance owner.
///
/// # Returns
/// The token balance of the owner as `i128`.
pub fn balance(env: &Env, token: &Address, owner: &Address) -> i128 {
    let client = get_token_client(env, token);
    client.balance(owner)
}

/// Safely queries the allowance granted by an owner to a spender.
///
/// # Arguments
/// * `env` - The current contract execution environment.
/// * `token` - The address of the token contract.
/// * `from` - The address of the token owner.
/// * `spender` - The address of the approved spender.
///
/// # Returns
/// The remaining allowance amount as `i128`.
pub fn allowance(env: &Env, token: &Address, from: &Address, spender: &Address) -> i128 {
    let client = get_token_client(env, token);
    client.allowance(from, spender)
}

/// Approves a spender to transfer tokens on behalf of the owner.
///
/// # Arguments
/// * `env` - The current contract execution environment.
/// * `token` - The address of the token contract.
/// * `from` - The address of the owner authorizing the allowance.
/// * `spender` - The address of the spender being authorized.
/// * `amount` - The amount to authorize.
/// * `expiration_ledger` - The ledger number at which this approval expires.
pub fn approve(
    env: &Env,
    token: &Address,
    from: &Address,
    spender: &Address,
    amount: &i128,
    expiration_ledger: u32,
) {
    if *amount < 0 {
        panic!("amount must be non-negative");
    }
    let client = get_token_client(env, token);
    client.approve(from, spender, amount, &expiration_ledger);
}

/// Initializes a token client and performs a transfer.
///
/// # Arguments
/// * `env` - The current contract execution environment.
/// * `token` - The address of the token contract.
/// * `from` - The sender's address.
/// * `to` - The recipient's address.
/// * `amount` - The token amount to transfer.
pub fn transfer(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
    if *amount <= 0 {
        panic!("transfer amount must be positive");
    }
    if from == to {
        panic!("sender and recipient addresses must be different");
    }
    let client = get_token_client(env, token);
    client.transfer(from, to, amount);
}

/// Transfers tokens using an approved allowance.
///
/// # Arguments
/// * `env` - The current contract execution environment.
/// * `token` - The address of the token contract.
/// * `spender` - The address authorized to execute the transfer.
/// * `from` - The address from which tokens are taken.
/// * `to` - The address receiving the tokens.
/// * `amount` - The token amount to transfer.
pub fn transfer_from(
    env: &Env,
    token: &Address,
    spender: &Address,
    from: &Address,
    to: &Address,
    amount: &i128,
) {
    if *amount <= 0 {
        panic!("transfer amount must be positive");
    }
    if from == to {
        panic!("sender and recipient addresses must be different");
    }
    let client = get_token_client(env, token);
    client.transfer_from(spender, from, to, amount);
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_get_token_client() {
        let env = Env::default();
        let token_addr = Address::generate(&env);
        let client = get_token_client(&env, &token_addr);
        assert_eq!(client.address, token_addr);
    }
}
