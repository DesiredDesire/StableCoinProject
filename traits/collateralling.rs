use brush::{
    contracts::psp22::*,
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type CollaterallingContractRef = dyn Collateralling + PSP22Receiver;

#[brush::wrapper]
pub type CollaterallingRef = dyn Collateralling;

#[brush::trait_definition]
pub trait Collateralling {
    #[ink(message)]
    fn collateral_amount(&self) -> Balance;

    #[ink(message)]
    fn get_collateral_token_address(&self) -> AccountId;
}

pub trait CollaterallingInternal {
    fn _transfer_collateral_in(
        &mut self,
        from: AccountId,
        amount: Balance,
    ) -> Result<(), PSP22Error>;
    fn _transfer_collateral_out(
        &mut self,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), PSP22Error>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CollaterallingError {
    PSP22Error(PSP22Error),
}

impl From<PSP22Error> for CollaterallingError {
    fn from(error: PSP22Error) -> Self {
        CollaterallingError::PSP22Error(error)
    }
}
