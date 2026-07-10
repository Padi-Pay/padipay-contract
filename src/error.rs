use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    NotFound = 1,
    InvalidAmount = 2,
    InvalidAddresses = 3,
    InvalidState = 4,
    AlreadyFunded = 5,
}
