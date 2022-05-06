use brush::{contracts::traits::ownable::*, traits::AccountId};

use super::feeding::FeedingError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type EatingContractRef = dyn Eating;

#[brush::wrapper]
pub type EatingRef = dyn Eating;

#[brush::trait_definition]
pub trait Eating {
    #[ink(message)]
    fn eat_collateral_price(&self) -> Result<u128, EatingError>;

    #[ink(message)]
    fn change_feeder(&mut self, new_vault_feeder_address: AccountId) -> Result<(), EatingError>;
}

pub trait EatingInternal {
    fn _emit_feeder_changed_event(
        &self,
        _old_feeder: Option<AccountId>,
        _new_feeder: Option<AccountId>,
        _caller: AccountId,
    );
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum EatingError {
    FeedingError(FeedingError),
    OwnableError(OwnableError),
}

impl From<FeedingError> for EatingError {
    fn from(error: FeedingError) -> Self {
        EatingError::FeedingError(error)
    }
}

impl From<OwnableError> for EatingError {
    fn from(error: OwnableError) -> Self {
        EatingError::OwnableError(error)
    }
}
