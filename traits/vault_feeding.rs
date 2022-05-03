#[brush::wrapper]
pub type VFeedingContractRef = dyn VFeeding;

#[brush::wrapper]
pub type VFeedingRef = dyn VFeeding;

#[brush::trait_definition]
pub trait VFeeding {
    #[ink(message)]
    fn feed_price(&mut self) -> Result<u128, VFeedingError>;

    #[ink(message)]
    fn feed_interest_rate(&mut self) -> Result<u128, VFeedingError>;

    #[ink(message)]
    fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VFeedingError>;

    #[ink(message)]
    fn feed_all(&mut self) -> Result<(u128, u128, u128), VFeedingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VFeedingError {
    CouldntFeed,
}
