use brush::{
    contracts::{traits::ownable::*, traits::psp22::PSP22Error},
    traits::AccountId,
};

use super::shares_profit_generating::SPGeneratingError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type SPControllingContractRef = dyn SPControlling + Ownable + SPControllingView;

#[brush::wrapper]
pub type PControllingRef = dyn SPControlling + SPControllingView;

#[brush::trait_definition]
pub trait SPControlling {
    // profitting and shares
    #[ink(message)]
    fn set_is_generator(&mut self, account: AccountId, is: bool) -> Result<(), SPControllingError>;

    // profitting
    #[ink(message)]
    fn collect_profit(&mut self, profit_generator: AccountId) -> Result<i128, SPControllingError>;

    #[ink(message)]
    fn distribute_income(&mut self) -> Result<(), SPControllingError>;

    #[ink(message)]
    fn set_treassury_address(
        &mut self,
        new_treassury_address: AccountId,
    ) -> Result<(), SPControllingError>;

    #[ink(message)]
    fn set_treassury_part_e6(
        &mut self,
        new_treassury_part_e6: u128,
    ) -> Result<(), SPControllingError>;

    // shares

    #[ink(message)]
    fn set_sharing_part_e6(
        &mut self,
        profit_generator: AccountId,
        new_sharing_part_e6: u128,
    ) -> Result<(), SPControllingError>;
}

#[brush::trait_definition]
pub trait SPControllingView {
    #[ink(message)]
    fn get_stable_coin_address(&self) -> AccountId;

    #[ink(message)]
    fn is_generator(&self, account: AccountId) -> bool;

    #[ink(message)]
    fn get_total_profit(&self) -> i128;

    #[ink(message)]
    fn get_treassury_address(&self) -> AccountId;

    #[ink(message)]
    fn get_treassury_part_e6(&self) -> u128;
}

pub trait SPControllingInternal {}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SPControllingError {
    Generator,
    NoProfit,
    One,
    PSP22Error(PSP22Error),
    OwnableError(OwnableError),
    SPGeneratingError(SPGeneratingError),
}

impl From<PSP22Error> for SPControllingError {
    fn from(error: PSP22Error) -> Self {
        SPControllingError::PSP22Error(error)
    }
}

impl From<OwnableError> for SPControllingError {
    fn from(error: OwnableError) -> Self {
        SPControllingError::OwnableError(error)
    }
}

impl From<SPGeneratingError> for SPControllingError {
    fn from(error: SPGeneratingError) -> Self {
        SPControllingError::SPGeneratingError(error)
    }
}
