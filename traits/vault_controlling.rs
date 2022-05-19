use super::measuring::*;
use super::vault::*;
use brush::contracts::traits::ownable::*;
use brush::traits::AccountId;

#[brush::wrapper]
pub type VControllingContractRef = dyn VControlling + VControllingView + Ownable;

#[brush::wrapper]
pub type VControllingRef = dyn VControlling + VControllingView;

#[brush::trait_definition]
pub trait VControlling {
    #[ink(message)]
    fn control_vault(&mut self) -> Result<(), VControllingError>;
}

#[brush::trait_definition]
pub trait VControllingView {
    #[ink(message)]
    fn get_vault_address(&self) -> AccountId;

    #[ink(message)]
    fn get_measurer_address(&self) -> AccountId;
}

pub trait VControllingInternal {
    fn _stability_measure_parameter_to_vault_parameters(
        &self,
        stability_measure: u8,
    ) -> (i16, u16, i16);
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VControllingError {
    CouldntFeed,
    MeasuringError(MeasuringError),
    VaultError(VaultError),
}

impl From<MeasuringError> for VControllingError {
    fn from(error: MeasuringError) -> Self {
        VControllingError::MeasuringError(error)
    }
}

impl From<VaultError> for VControllingError {
    fn from(error: VaultError) -> Self {
        VControllingError::VaultError(error)
    }
}
