use brush::{
    contracts::{psp22::PSP22Error, traits::pausable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type EmittingContractRef = dyn Emitting + Pausable;

#[brush::wrapper]
pub type EmittingRef = dyn Emitting;

#[brush::trait_definition]
pub trait Emitting {
    #[ink(message)]
    fn emited_amount(&self) -> Balance;

    #[ink(message)]
    fn get_emited_token_address(&self) -> AccountId;
}

pub trait EmittingInternal {
    fn _mint_emited_token(&mut self, to: AccountId, amount: Balance) -> Result<(), EmittingError>;
    fn _burn_emited_token(&mut self, from: AccountId, amount: Balance)
        -> Result<(), EmittingError>;
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
