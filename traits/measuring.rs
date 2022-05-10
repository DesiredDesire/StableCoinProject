//TODO use brush::contracts::traits::access_control::*;
use brush::contracts::traits::pausable::*;

#[brush::wrapper]
pub type MeasuringContractRef = dyn Measuring + Pausable;

#[brush::wrapper]
pub type MeasuringRef = dyn Measuring;

#[brush::trait_definition]
pub trait Measuring {
    // can be triggered once per some defnied period of time
    #[ink(message)]
    fn update_stability_measure_parameter(&mut self) -> Result<u8, MeasuringError>;

    #[ink(message)]
    fn get_stability_measure_parameter(&self) -> u8; //(stability_measure_parameter: u8, block_timestamp: u32)
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MeasuringError {
    CouldntFeed,
    PausableError(PausableError),
}

impl From<PausableError> for MeasuringError {
    fn from(error: PausableError) -> Self {
        MeasuringError::PausableError(error)
    }
}
