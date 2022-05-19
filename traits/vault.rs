use brush::{
    contracts::{
        ownable::OwnableError, pausable::PausableError, psp22::PSP22Error, psp34::PSP34Error,
        traits::ownable::*, traits::pausable::*,
    },
    traits::{AccountId, Balance},
};

use crate::traits::collateralling::*;
use crate::traits::emitting::*;
use crate::traits::shares_profit_generating::*;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type VaultContractRef =
    dyn Vault + VaultView + Ownable + Pausable + Collateralling + Emitting + SPGenerating; //TODO

#[brush::trait_definition]
pub trait VaultContractCheck:
    Vault + VaultView + Ownable + Pausable + Collateralling + Emitting + SPGenerating
{
}

#[brush::wrapper]
pub type VaultRef = dyn Vault + VaultView;

//
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
    fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn buy_risky_vault(&mut self, vault_id: u128) -> Result<(), VaultError>;
    #[ink(message)]
    fn be_controlled(
        &mut self,
        interest_rate_step: i16,
        collateral_step: u16,
        stable_coin_interest_rate_step: i16,
    ) -> Result<(), VaultError>;
    #[ink(message)]
    fn set_vault_controller_address(
        &mut self,
        controller_address: AccountId,
    ) -> Result<(), VaultError>;
    #[ink(message)]
    fn set_oracle_address(&mut self, new_oracle_address: AccountId) -> Result<(), VaultError>;
    #[ink(message)]
    fn set_liquidator_address(
        &mut self,
        new_liquidator_address: AccountId,
    ) -> Result<(), VaultError>;
}

#[brush::trait_definition]
pub trait VaultView {
    #[ink(message)]
    fn get_next_id(&mut self) -> u128;
    #[ink(message)]
    fn get_total_debt(&self) -> Balance;
    #[ink(message)]
    fn get_vault_details(&self, vault_id: u128) -> (Balance, Balance);
    #[ink(message)]
    fn get_vault_controller_address(&self) -> AccountId;
    #[ink(message)]
    fn get_oracle_address(&self) -> AccountId;
    #[ink(message)]
    fn get_debt_ceiling(&self, vault_id: u128) -> Balance;
    #[ink(message)]
    fn get_liquidator_address(&self) -> AccountId;
}
pub trait VaultInternal {
    fn _emit_deposit_event(&self, _vault_id: u128, _current_collateral: Balance);
    fn _emit_withdraw_event(&self, _vault_id: u128, _current_collateral: Balance);
    fn _emit_borrow_event(&self, _vault_id: u128, _borrowed: Balance);
    fn _emit_pay_back_event(&self, _vault_id: u128, _pay_backed: Balance);
    fn _get_debt_ceiling(&self, vault_id: u128) -> Balance;
    fn _collateral_value_e6(&self, collateral: Balance) -> u128;
    fn _vault_collateral_value_e6(&self, value_id: u128) -> u128;
    fn _update_vault_debt(&mut self, vault_id: u128) -> Result<Balance, VaultError>;
    fn _update_current_interest_coefficient_e12(&mut self) -> u128;
    fn _get_current_interest_coefficient_e12(&self) -> u128;
    fn _get_debt_by_id(&self, vault_id: &u128) -> Balance;
    fn _get_collateral_by_id(&self, vault_id: &u128) -> Balance;
    fn _get_last_interest_coefficient_by_id_e12(&self, vault_id: &u128) -> Balance;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VaultError {
    VaultController,
    OwnerUnexists,
    DebtUnexists,
    CollateralUnexists,
    HasDebt,
    NotEmpty,
    VaultOwnership,
    CollateralBelowMinimum,
    CollateralAboveMinimum,
    Liquidator,
    PSP22Error(PSP22Error),
    PSP34Error(PSP34Error),
    PausableError(PausableError),
    CollaterallingError(CollaterallingError),
    OwnableError(OwnableError),
    EmittingError(EmittingError),
}

impl From<PSP22Error> for VaultError {
    fn from(error: PSP22Error) -> Self {
        VaultError::PSP22Error(error)
    }
}

impl From<PSP34Error> for VaultError {
    fn from(error: PSP34Error) -> Self {
        VaultError::PSP34Error(error)
    }
}

impl From<OwnableError> for VaultError {
    fn from(error: OwnableError) -> Self {
        VaultError::OwnableError(error)
    }
}

impl From<PausableError> for VaultError {
    fn from(error: PausableError) -> Self {
        VaultError::PausableError(error)
    }
}

impl From<EmittingError> for VaultError {
    fn from(error: EmittingError) -> Self {
        VaultError::EmittingError(error)
    }
}
impl From<CollaterallingError> for VaultError {
    fn from(error: CollaterallingError) -> Self {
        VaultError::CollaterallingError(error)
    }
}
