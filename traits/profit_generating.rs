use brush::{
    contracts::traits::ownable::*,
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type PGeneratingContractRef = dyn PGenerating + Ownable;

#[brush::wrapper]
pub type PGeneratingRef = dyn PGenerating;

#[brush::trait_definition]
pub trait PGenerating {
    #[ink(message)]
    fn give_profit(&mut self) -> Result<i128, PGeneratingError>;

    #[ink(message)]
    fn set_profit_controller_address(
        &mut self,
        new_profit_controller: AccountId,
    ) -> Result<(), PGeneratingError>;
}

#[brush::trait_definition]
pub trait PGeneratingView {
    #[ink(message)]
    fn amount_to_collect(&self) -> i128;

    #[ink(message)]
    fn get_profit_controller_address(&self) -> AccountId;
}
pub trait PGeneratingInternal {
    fn _add_profit(&mut self, amount: Balance);
    fn _sub_profit(&mut self, amount: Balance);
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PGeneratingError {
    Controller,
    OwnableError(OwnableError),
}

impl From<OwnableError> for PGeneratingError {
    fn from(error: OwnableError) -> Self {
        PGeneratingError::OwnableError(error)
    }
}
