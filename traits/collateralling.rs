use brush::{
    contracts::{psp22::*, traits::ownable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type CollaterallingContractRef = dyn Collateralling + Ownable + PSP22Receiver;

#[brush::wrapper]
pub type CollaterallingRef = dyn Collateralling;

#[brush::trait_definition]
pub trait Collateralling {
    #[ink(message)]
    fn collateral_amount(&self) -> Balance;

    #[ink(message)]
    fn rescue_psp22(
        &mut self,
        psp22_address: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), CollaterallingError>;

    #[ink(message)]
    fn get_collateral_token_address(&self) -> AccountId;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CollaterallingError {
    OwnableError(OwnableError),
    PSP22Error(PSP22Error),
    PSP22ReceiverError(PSP22ReceiverError),
}

impl From<OwnableError> for CollaterallingError {
    fn from(error: OwnableError) -> Self {
        CollaterallingError::OwnableError(error)
    }
}

impl From<PSP22Error> for CollaterallingError {
    fn from(error: PSP22Error) -> Self {
        CollaterallingError::PSP22Error(error)
    }
}

impl From<PSP22ReceiverError> for CollaterallingError {
    fn from(error: PSP22ReceiverError) -> Self {
        CollaterallingError::PSP22ReceiverError(error)
    }
}
