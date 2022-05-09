use brush::contracts::traits::ownable::*;

use super::system::SystemError;

#[brush::wrapper]
pub type VControllingContractRef = dyn VControlling;

#[brush::wrapper]
pub type VControllingRef = dyn VControlling;

#[brush::trait_definition]
pub trait VControlling {
    #[ink(message)]
    fn get_maximum_minimum_collateral_e6(&mut self) -> u128;

    #[ink(message)]
    fn get_vault_parameters(&mut self) -> (u128, u128, u128);

    #[ink(message)]
    fn set_vault_parameters(
        &mut self,
        interest_rate_step_e12: u128,
        minimum_collateral_coef_step_e6: u128,
        stable_coin_interest_rate_step_e12: u128,
    ) -> Result<(), VControllingError>;

    #[ink(message)]
    fn update_stability_measure_parameter(&mut self) -> Result<u8, VControllingError>;

    #[ink(message)]
    fn get_stability_measure_parameter(&self) -> u8;

    #[ink(message)]
    fn feed_interest_rate(&mut self) -> Result<u128, VControllingError>;

    #[ink(message)]
    fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VControllingError>;

    #[ink(message)]
    fn feed_all(&mut self) -> Result<(u128, u128, u128), VControllingError>;
}

pub trait VControllingInternal {
    fn _update_stability_measure_parameter(&mut self) -> Result<u8, VControllingError>;
    fn _get_stability_measure_parameter(&self) -> u8;
    fn _stability_measure_parameter_to_vault_parameters(
        &self,
        state_parameter: u8,
    ) -> Result<(u128, u128, u128), VControllingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VControllingError {
    CouldntFeed,
    OwnableError(OwnableError),
    SystemError(SystemError),
}

impl From<OwnableError> for VControllingError {
    fn from(error: OwnableError) -> Self {
        VControllingError::OwnableError(error)
    }
}

impl From<SystemError> for VControllingError {
    fn from(error: SystemError) -> Self {
        VControllingError::SystemError(error)
    }
}
