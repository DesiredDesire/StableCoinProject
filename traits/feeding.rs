#[brush::wrapper]
pub type FeedingContractRef = dyn Feeding;

#[brush::wrapper]
pub type FeedingRef = dyn Feeding;

#[brush::trait_definition]
pub trait Feeding {
    #[ink(message)]
    fn feed_price(&mut self) -> Result<u128, FeedingError>;
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FeedingError {
    CouldntFeed,
}
