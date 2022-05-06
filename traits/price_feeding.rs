use brush::contracts::traits::ownable::*;

#[brush::wrapper]
pub type PFeedingContractRef = dyn PFeeding;

#[brush::wrapper]
pub type PFeedingRef = dyn PFeeding;

#[brush::trait_definition]
pub trait PFeeding {
    #[ink(message)]
    fn feed_price(&mut self) -> Result<u128, PFeedingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PFeedingError {
    CouldntFeed,
    OwnableError(OwnableError),
}

impl From<OwnableError> for PFeedingError {
    fn from(error: OwnableError) -> Self {
        PFeedingError::OwnableError(error)
    }
}
