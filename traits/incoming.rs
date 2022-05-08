use brush::{
    contracts::{psp22::PSP22Error, traits::pausable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type IncomingContractRef = dyn Incoming + Ownable;

#[brush::wrapper]
pub type IncomingRef = dyn Incoming;

#[brush::trait_definition]
pub trait Incoming {
    #[ink(message)]
    fn amount_to_collect(&self) -> Balance;

    #[ink(message)]
    fn collect(&self) -> Balance;

    #[ink(message)]
    fn get_collected_token_address(&self) -> AccountId;

    #[ink(message)]
    fn get_treassury_address(&self) -> AccountId;

    #[ink(message)]
    fn change_treassury_address(&self);
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum EmittingError {
    PausableError(PausableError),

    CouldntMint,
    PSP22Error(PSP22Error),
}

impl From<PausableError> for EmittingError {
    fn from(error: PausableError) -> Self {
        EmittingError::PausableError(error)
    }
}

impl From<PSP22Error> for EmittingError {
    fn from(error: PSP22Error) -> Self {
        EmittingError::PSP22Error(error)
    }
}
