use brush::contracts::traits::ownable::*;

use super::system::SystemError;

#[brush::wrapper]
pub type SControllingContractRef = dyn SControlling;

#[brush::wrapper]
pub type SControllingRef = dyn SControlling;

#[brush::trait_definition]
pub trait SControlling {
    #[ink(message)]
    fn update_stability_measure_parameter(&mut self) -> Result<u8, SControllingError>;

    #[ink(message)]
    fn get_stability_measure_parameter(&self) -> u8;

    #[ink(message)]
    fn update_current_denominator_e12(&mut self) -> Result<u128, SControllingError>;

    #[ink(message)]
    fn get_current_denominator_e12(&self) -> u128;
}

pub trait SControllingInternal {
    fn _update_stability_measure_parameter(&mut self) -> Result<u8, SControllingError>;
    fn _get_stability_measure_parameter(&self) -> u8;
    fn _stability_measure_parameter_to_interest_rate(
        &self,
        state_parameter: u8,
    ) -> Result<i128, SControllingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SControllingError {
    CouldntFeed,
    OwnableError(OwnableError),
    SystemError(SystemError),
}

impl From<OwnableError> for SControllingError {
    fn from(error: OwnableError) -> Self {
        SControllingError::OwnableError(error)
    }
}

impl From<SystemError> for SControllingError {
    fn from(error: SystemError) -> Self {
        SControllingError::SystemError(error)
    }
}
