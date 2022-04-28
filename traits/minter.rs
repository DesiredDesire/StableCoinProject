use brush::{
    contracts::{psp22::PSP22Error, traits::pausable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type MinterContractRef = dyn Minter + Pausable;

#[brush::wrapper]
pub type MinterRef = dyn Minter;

#[brush::trait_definition]
pub trait Minter {
    #[ink(message)]
    fn mint(&mut self, to: AccountId, amount: Balance) -> Result<(), MinterError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MinterError {
    PausableError(PausableError),

    CouldntMint,
    PSP22Error(PSP22Error),
}

impl From<PausableError> for MinterError {
    fn from(error: PausableError) -> Self {
        MinterError::PausableError(error)
    }
}

impl From<PSP22Error> for MinterError {
    fn from(error: PSP22Error) -> Self {
        MinterError::PSP22Error(error)
    }
}
