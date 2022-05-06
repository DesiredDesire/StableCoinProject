use brush::contracts::traits::ownable::*;

#[brush::wrapper]
pub type VFeedingContractRef = dyn VFeeding;

#[brush::wrapper]
pub type VFeedingRef = dyn VFeeding;

#[brush::trait_definition]
pub trait VFeeding {
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
    ) -> Result<(), VFeedingError>;
    #[ink(message)]
    fn get_state_parameter(&mut self) -> Result<u8, VFeedingError>;

    #[ink(message)]
    fn feed_interest_rate(&mut self) -> Result<u128, VFeedingError>;

    #[ink(message)]
    fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VFeedingError>;

    #[ink(message)]
    fn feed_all(&mut self) -> Result<(u128, u128, u128), VFeedingError>;
}

pub trait VFeedingInternal {
    fn _get_state_parameter(&self) -> Result<u8, VFeedingError>;
    fn _state_parameter_to_vault_parameters(
        &self,
        state_parameter: u8,
    ) -> Result<(u128, u128, u128), VFeedingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VFeedingError {
    CouldntFeed,
    OwnableError(OwnableError),
}

impl From<OwnableError> for VFeedingError {
    fn from(error: OwnableError) -> Self {
        VFeedingError::OwnableError(error)
    }
}
