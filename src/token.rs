use soroban_sdk::{token, Address, Env};

/// Creates a token client for the specified token contract address.
pub fn get_token_client<'a>(env: &'a Env, token: &Address) -> token::Client<'a> {
    token::Client::new(env, token)
}

/// Initializes a token client and performs a transfer.
pub fn transfer(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
    let client = get_token_client(env, token);
    client.transfer(from, to, amount);
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

