use brush::contracts::traits::ownable::*;
use brush::contracts::traits::psp22::*;

use super::measuring::*;

#[brush::wrapper]
pub type SControllingContractRef = dyn SControlling;

#[brush::wrapper]
pub type SControllingRef = dyn SControlling;

#[brush::trait_definition]
pub trait SControlling {
    #[ink(message)]
    fn control_stable_coin(&mut self) -> Result<(), SControllingError>;
}

pub trait SControllingInternal {
    fn _stability_measure_parameter_to_interest_rate(&self, state_parameter: u8) -> i128;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SControllingError {
    CouldntFeed,
    OwnableError(OwnableError),
    MeasuringError(MeasuringError),
    PSP22Error(PSP22Error),
}

impl From<OwnableError> for SControllingError {
    fn from(error: OwnableError) -> Self {
        SControllingError::OwnableError(error)
    }
}

impl From<MeasuringError> for SControllingError {
    fn from(error: MeasuringError) -> Self {
        SControllingError::MeasuringError(error)
    }
}

impl From<PSP22Error> for SControllingError {
    fn from(error: PSP22Error) -> Self {
        SControllingError::PSP22Error(error)
    }
}
