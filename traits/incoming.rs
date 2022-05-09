use brush::{
    contracts::{psp22::PSP22Error, traits::ownable::*, traits::pausable::*},
    traits::{AccountId, Balance},
};

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type IncomingContractRef = dyn Incoming + Ownable;

#[brush::wrapper]
pub type IncomingRef = dyn Incoming;

#[brush::trait_definition]
pub trait Incoming {
    #[ink(message)]
    fn amount_to_collect(&self) -> Balance;

    #[ink(message)]
    fn collect(&self) -> Result<Balance, PausableError>;

    #[ink(message)]
    fn get_collected_token_address(&self) -> AccountId;

    #[ink(message)]
    fn get_treassury_address(&self) -> AccountId;

    #[ink(message)]
    fn change_treassury_address(&self) -> Result<(), OwnableError>;
}
