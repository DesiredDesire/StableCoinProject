use brush::{
    contracts::{psp22::PSP22Error, traits::pausable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type EmitingContractRef = dyn Emiting + Pausable;

#[brush::wrapper]
pub type EmitingRef = dyn Emiting;

#[brush::trait_definition]
pub trait Emiting {
    #[ink(message)]
    fn emited_amount(&self) -> Balance;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum EmitingError {
    PausableError(PausableError),

    CouldntMint,
    PSP22Error(PSP22Error),
}

impl From<PausableError> for EmitingError {
    fn from(error: PausableError) -> Self {
        EmitingError::PausableError(error)
    }
}

impl From<PSP22Error> for EmitingError {
    fn from(error: PSP22Error) -> Self {
        EmitingError::PSP22Error(error)
    }
}
