use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    InvalidState = 2,
    EscrowNotFound = 3,
    InvalidAmount = 4,
    EscrowAlreadyFunded = 5,
    InvalidAddresses = 6,
}
