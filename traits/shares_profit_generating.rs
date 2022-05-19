use brush::{
    contracts::{psp22::PSP22Error, traits::ownable::*, traits::pausable::*},
    traits::{AccountId, Balance},
};

#[brush::wrapper]
pub type SPGeneratingContractRef = dyn SPGenerating + SPGeneratingView + Ownable;

#[brush::wrapper]
pub type SPGeneratingRef = dyn SPGenerating + SPGeneratingView;

#[brush::trait_definition]
pub trait SPGenerating {
    // profiting and shares
    #[ink(message)]
    fn set_sharing_part_e6(&mut self, new_sharing_part_e6: u128) -> Result<(), SPGeneratingError>;

    // profitting
    #[ink(message)]
    fn give_profit(&mut self) -> Result<i128, SPGeneratingError>;

    #[ink(message)]
    fn set_shares_profit_controller_address(
        &mut self,
        new_profit_controller: AccountId,
    ) -> Result<(), SPGeneratingError>;

    // shares
    fn mint_shares(&mut self) -> Result<(), SPGeneratingError>;
}

#[brush::trait_definition]
pub trait SPGeneratingView {
    // profiting and shares
    #[ink(message)]
    fn get_sharing_part_e6(&self) -> u128;

    // profiting
    #[ink(message)]
    fn get_generated_profit(&self) -> i128;

    #[ink(message)]
    fn get_shares_profit_controller_address(&self) -> AccountId;

    // shares
    #[ink(message)]
    fn get_shares_token_address(&self) -> AccountId;

    #[ink(message)]
    fn get_shares_minting_allowance(&self, account: AccountId) -> u128;
}

pub trait SPGeneratingInternal {
    // profiting and shares
    fn _add_profit_and_increase_shares_minting_allowance(
        &mut self,
        amount: u128,
        account: AccountId,
    );
    // profitng
    fn _add_profit(&mut self, amount: Balance);
    fn _sub_profit(&mut self, amount: Balance);

    //shares
    fn _mint_shares(&mut self, to: AccountId) -> Result<(), SPGeneratingError>;
    fn _get_shares_minting_allowance(&self, account: AccountId) -> u128;
    fn _increase_shares_minting_allowance(&mut self, account: AccountId, amount: Balance);
}

/// Enum of errors raised by our lending smart contract
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SPGeneratingError {
    PausableError(PausableError),
    PSP22Error(PSP22Error),
    OwnableError(OwnableError),
    Controller,
}

impl From<PausableError> for SPGeneratingError {
    fn from(error: PausableError) -> Self {
        SPGeneratingError::PausableError(error)
    }
}

impl From<PSP22Error> for SPGeneratingError {
    fn from(error: PSP22Error) -> Self {
        SPGeneratingError::PSP22Error(error)
    }
}

impl From<OwnableError> for SPGeneratingError {
    fn from(error: OwnableError) -> Self {
        SPGeneratingError::OwnableError(error)
    }
}
