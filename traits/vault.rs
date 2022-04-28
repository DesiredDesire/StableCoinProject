use brush::{
    contracts::{
        psp34::{Id, PSP34Error},
        traits::pausable::*,
    },
    traits::Balance,
};

use super::minter::MinterError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type VaultContractRef = dyn Vault + Pausable;

#[brush::wrapper]
pub type VaultRef = dyn Vault;

#[brush::trait_definition]
pub trait Vault {
    #[ink(message)]
    fn deposit(&mut self, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn withdraw(&mut self, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn create_vault(&mut self) -> Result<Id, VaultError>;
    #[ink(message)]
    fn get_debt_ceiling(&mut self) -> Balance;
    #[ink(message)]
    fn destroy_vault(&mut self, vault_id: Id) -> Result<(), VaultError>;
    #[ink(message)]
    fn transfer_vault(&mut self, vault_id: Id, address_to: Id) -> ();
    #[ink(message)]
    fn deposit_collateral(&mut self, vault_id: Id, amount: Balance) -> ();
    #[ink(message)]
    fn withdraw_collateral(&mut self, vault_id: Id, amount: Balance) -> ();
    #[ink(message)]
    fn borrow_token(&mut self, vault_id: Id, amount: Balance) -> ();
    #[ink(message)]
    fn pay_back_token(&mut self, vault_id: Id, amount: Balance) -> ();
    #[ink(message)]
    fn buy_risky_vault(&mut self, vault_id: Id) -> ();
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum WithdrawError {
    InsufficientCollateralError,
    CollateralCriticalAmountError,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CollateralError {
    PriceEqualsZeroError,
    CollateralBelowMinimumPercentageError,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VaultError {
    CollateralError(CollateralError),
    NonExistingVaultError,
    VaultOwnershipError,
    WithdrawError(WithdrawError),
    DepositError,
    PSP34Error(PSP34Error),
    MinterError(MinterError),
}

impl From<PSP34Error> for VaultError {
    fn from(error: PSP34Error) -> Self {
        VaultError::PSP34Error(error)
    }
}

impl From<MinterError> for VaultError {
    fn from(error: MinterError) -> Self {
        VaultError::MinterError(error)
    }
}

impl From<WithdrawError> for VaultError {
    fn from(error: WithdrawError) -> Self {
        VaultError::WithdrawError(error)
    }
}

impl From<CollateralError> for VaultError {
    fn from(error: CollateralError) -> Self {
        VaultError::CollateralError(error)
    }
}
