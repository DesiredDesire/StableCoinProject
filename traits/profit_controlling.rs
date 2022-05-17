use brush::{
    contracts::{traits::ownable::*, traits::pausable::*},
    traits::AccountId,
};

use super::emitting::EmittingError;
use super::profit_generating::PGeneratingError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type PControllingContractRef = dyn PControlling + Ownable + Pausable;

#[brush::wrapper]
pub type PControllingRef = dyn PControlling;

#[brush::trait_definition]
pub trait PControlling {
    #[ink(message)]
    fn collect_profit(&mut self, profit_generator: AccountId) -> Result<i128, PControllingError>;

    #[ink(message)]
    fn distribute_income(&mut self) -> Result<(), PControllingError>;

    #[ink(message)]
    fn set_treassury_address(
        &mut self,
        new_treassury_address: AccountId,
    ) -> Result<(), PControllingError>;

    #[ink(message)]
    fn set_treassury_part_e6(
        &mut self,
        new_treassury_part_e6: u128,
    ) -> Result<(), PControllingError>;
}

#[brush::trait_definition]
pub trait PControllingView {
    #[ink(message)]
    fn get_total_profit(&self) -> i128;

    #[ink(message)]
    fn get_treassury_address(&self) -> AccountId;
}

pub trait PControllingInternal {}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PControllingError {
    Generator,
    NoProfit,
    One,
    OwnableError(OwnableError),
    EmittingError(EmittingError),
    PGeneratingError(PGeneratingError),
}

impl From<OwnableError> for PControllingError {
    fn from(error: OwnableError) -> Self {
        PControllingError::OwnableError(error)
    }
}

impl From<EmittingError> for PControllingError {
    fn from(error: EmittingError) -> Self {
        PControllingError::EmittingError(error)
    }
}

impl From<PGeneratingError> for PControllingError {
    fn from(error: PGeneratingError) -> Self {
        PControllingError::PGeneratingError(error)
    }
}
