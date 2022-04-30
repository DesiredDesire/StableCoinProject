use brush::{
    contracts::{psp22::PSP22Error, psp34::PSP34Error, traits::pausable::*},
    traits::Balance,
};

use super::emiting::EmitingError;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type VaultContractRef = dyn Vault + Pausable;

#[brush::wrapper]
pub type VaultRef = dyn Vault;

#[brush::trait_definition]
pub trait Vault {
    #[ink(message)]
    fn create_vault(&mut self) -> Result<(), VaultError>;
    #[ink(message)]
    fn destroy_vault(&mut self, vault_id: u128) -> Result<(), VaultError>;
    #[ink(message)]
    fn deposit_collateral(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn withdraw_collateral(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn get_debt_ceiling(&mut self, vault_id: u128) -> Result<Balance, VaultError>;
    #[ink(message)]
    fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn buy_risky_vault(&mut self, vault_id: u128) -> ();
}
pub trait VaultInternal {
    fn _check_collateral(&mut self, collateral: Balance, debt: Balance) -> Result<(), VaultError>;
    fn _collateral_value_e6(&mut self, collateral: Balance) -> Result<Balance, VaultError>;
    fn _collateral_value_e6_view(&self, collateral: Balance) -> Result<Balance, VaultError>;
    fn _vault_collateral_value_e6(&mut self, value_id: u128) -> Result<Balance, VaultError>;
    fn _vault_collateral_value_e6_view(&self, value_id: u128) -> Result<Balance, VaultError>;
    fn _update_collateral_price(&mut self) -> Result<u128, VaultError>;
    fn _get_collateral_price(&self) -> Result<u128, VaultError>;
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
    Unexists,
    Exists,
    HasDebt,
    NotEmpty,
    VaultOwnership,
    CollateralBelowMinimumPercentage,
    WithdrawError(WithdrawError),
    DepositError,
    PSP34Error(PSP34Error),
    PSP22Error(PSP22Error),
    EmitingError(EmitingError),
}

impl From<PSP34Error> for VaultError {
    fn from(error: PSP34Error) -> Self {
        VaultError::PSP34Error(error)
    }
}

impl From<PSP22Error> for VaultError {
    fn from(error: PSP22Error) -> Self {
        VaultError::PSP22Error(error)
    }
}

impl From<EmitingError> for VaultError {
    fn from(error: EmitingError) -> Self {
        VaultError::EmitingError(error)
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
