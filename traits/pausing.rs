use brush::contracts::{traits::ownable::*, traits::pausable::*};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type PausingContractRef = dyn Pausing + Pausable;

#[brush::wrapper]
pub type PausingRef = dyn Pausing;

#[brush::trait_definition]
pub trait Pausing {
    #[ink(message)]
    fn pause(&mut self) -> Result<(), PausingError>;

    #[ink(message)]
    fn unpause(&mut self) -> Result<(), PausingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PausingError {
    OwnableError(OwnableError),
    PausableError(PausableError),
}

impl From<OwnableError> for PausingError {
    fn from(error: OwnableError) -> Self {
        PausingError::OwnableError(error)
    }
}

impl From<PausableError> for PausingError {
    fn from(error: PausableError) -> Self {
        PausingError::PausableError(error)
    }
}
