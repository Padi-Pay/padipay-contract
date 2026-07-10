use soroban_sdk::{contracttype, Address};

/// Represents the lifecycle states of an escrow.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// The escrow has been initialized but funds are not yet locked.
    Created,
    /// Funds have been locked in the escrow contract.
    Locked,
    /// Funds have been successfully released to the seller.
    Released,
    /// Funds have been returned to the buyer.
    Refunded,
}

impl EscrowStatus {
    /// Validates whether a state transition is legal.
    pub fn is_valid_transition(&self, next: &EscrowStatus) -> bool {
        matches!(
            (self, next),
            (EscrowStatus::Created, EscrowStatus::Locked)
                | (EscrowStatus::Locked, EscrowStatus::Released)
                | (EscrowStatus::Locked, EscrowStatus::Refunded)
        )
    }
}

/// Storage keys for the contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The global administrator or mediator of the contract.
    Admin,
    /// The escrow state associated with this contract instance.
    State,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(EscrowStatus::Created.is_valid_transition(&EscrowStatus::Locked));
        assert!(EscrowStatus::Locked.is_valid_transition(&EscrowStatus::Released));
        assert!(EscrowStatus::Locked.is_valid_transition(&EscrowStatus::Refunded));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!EscrowStatus::Created.is_valid_transition(&EscrowStatus::Released));
        assert!(!EscrowStatus::Created.is_valid_transition(&EscrowStatus::Refunded));
        assert!(!EscrowStatus::Created.is_valid_transition(&EscrowStatus::Created));

        assert!(!EscrowStatus::Locked.is_valid_transition(&EscrowStatus::Locked));
        assert!(!EscrowStatus::Locked.is_valid_transition(&EscrowStatus::Created));

        assert!(!EscrowStatus::Released.is_valid_transition(&EscrowStatus::Created));
        assert!(!EscrowStatus::Released.is_valid_transition(&EscrowStatus::Locked));
        assert!(!EscrowStatus::Released.is_valid_transition(&EscrowStatus::Refunded));
        assert!(!EscrowStatus::Released.is_valid_transition(&EscrowStatus::Released));

        assert!(!EscrowStatus::Refunded.is_valid_transition(&EscrowStatus::Created));
        assert!(!EscrowStatus::Refunded.is_valid_transition(&EscrowStatus::Locked));
        assert!(!EscrowStatus::Refunded.is_valid_transition(&EscrowStatus::Released));
        assert!(!EscrowStatus::Refunded.is_valid_transition(&EscrowStatus::Refunded));
    }
}

/// Represents the state of an escrow agreement.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowState {
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
}
