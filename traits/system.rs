use brush::contracts::traits::ownable::*;

#[brush::wrapper]
pub type SystemContractRef = dyn System;

#[brush::wrapper]
pub type SystemRef = dyn System;

#[brush::trait_definition]
pub trait System {
    #[ink(message)]
    fn update_stablecoin_interest_rate(&mut self) -> Result<u128, SystemError>; // TODO move to stablecoin feeder

    #[ink(message)]
    fn get_stablecoin_interest_rate(&self) -> u128;

    #[ink(message)]
    fn update_stability_measure_parameter(&mut self) -> Result<u8, SystemError>;
}

pub trait SystemInternal {}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SystemError {
    CouldntFeed,
    OwnableError(OwnableError),
}

impl From<OwnableError> for SystemError {
    fn from(error: OwnableError) -> Self {
        SystemError::OwnableError(error)
    }
}
