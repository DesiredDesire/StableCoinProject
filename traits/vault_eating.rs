use brush::{contracts::traits::ownable::*, traits::AccountId};

use super::price_feeding::PFeedingError;
use super::vault_controlling::VControllingError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type VEatingContractRef = dyn VEating;

#[brush::wrapper]
pub type VEatingRef = dyn VEating;

#[brush::trait_definition]
pub trait VEating {
    #[ink(message)]
    fn eat_collateral_price_e6(&self) -> Result<u128, VEatingError>;

    #[ink(message)]
    fn eat_interest_rate_e12(&self) -> Result<u128, VEatingError>;

    #[ink(message)]
    fn eat_minimum_collateral_coefficient_e6(&self) -> Result<u128, VEatingError>;

    #[ink(message)]
    fn eat_all(&self) -> Result<(u128, u128, u128), VEatingError>;

    #[ink(message)]
    fn change_feeder(
        &mut self,
        new_vault_controller_address: AccountId,
    ) -> Result<(), VEatingError>;

    #[ink(message)]
    fn get_vault_controller_address(&self) -> AccountId;
}

pub trait VEatingInternal {
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
pub enum VEatingError {
    VControllingError(VControllingError),
    PFeedingError(PFeedingError),
    OwnableError(OwnableError),
}

impl From<VControllingError> for VEatingError {
    fn from(error: VControllingError) -> Self {
        VEatingError::VControllingError(error)
    }
}

impl From<PFeedingError> for VEatingError {
    fn from(error: PFeedingError) -> Self {
        VEatingError::PFeedingError(error)
    }
}

impl From<OwnableError> for VEatingError {
    fn from(error: OwnableError) -> Self {
        VEatingError::OwnableError(error)
    }
}
