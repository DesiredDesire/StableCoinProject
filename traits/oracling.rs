use brush::contracts::traits::access_control::*;
use brush::contracts::traits::ownable::*;

#[brush::wrapper]
pub type OraclingContractRef = dyn Oracling + AccessControl + Ownable;

#[brush::wrapper]
pub type OraclingRef = dyn Oracling;

#[brush::trait_definition]
pub trait Oracling {
    #[ink(message)]
    fn get_azero_usd_price_e6(&self) -> u128;

    #[ink(message)]
    fn get_azero_ausd_price_e6(&self) -> u128;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum OraclingError {
    CouldntFeed,
    OwnableError(OwnableError),
    AccessControlError(AccessControlError),
}

impl From<OwnableError> for OraclingError {
    fn from(error: OwnableError) -> Self {
        OraclingError::OwnableError(error)
    }
}

impl From<AccessControlError> for OraclingError {
    fn from(error: AccessControlError) -> Self {
        OraclingError::AccessControlError(error)
    }
}
