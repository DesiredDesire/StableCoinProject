use brush::{
    contracts::{
        ownable::OwnableError, pausable::PausableError, psp34::PSP34Error, traits::ownable::*,
        traits::pausable::*,
    },
    traits::Balance,
};

use crate::traits::collateralling::*;
use crate::traits::emitting::*;
use crate::traits::vault_eating::*;

/// Combination of all traits of the contract to simplify calls to the contract
#[brush::wrapper]
pub type VaultContractRef = dyn Vault + Ownable + Pausable + Collateralling + Emitting + VEating;

#[brush::trait_definition]
pub trait VaultContractCheck:
    Vault + Ownable + Pausable + Collateralling + Emitting + VEating
{
}

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
    fn get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError>;
    #[ink(message)]
    fn borrow_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn pay_back_token(&mut self, vault_id: u128, amount: Balance) -> Result<(), VaultError>;
    #[ink(message)]
    fn buy_risky_vault(&mut self, vault_id: u128) -> Result<(), VaultError>;
}
pub trait VaultInternal {
    fn _emit_deposit_event(&self, _vault_id: u128, _current_collateral: Balance);
    fn _emit_withdraw_event(&self, _vault_id: u128, _current_collateral: Balance);
    fn _emit_borrow_event(&self, _vault_id: u128, _current_debt: Balance);
    fn _emit_pay_back_event(&self, _vault_id: u128, _current_debt: Balance);
    fn _get_debt_ceiling(&self, vault_id: u128) -> Result<Balance, VaultError>;
    fn _collateral_value_e6(&self, collateral: Balance) -> Result<Balance, VaultError>;
    fn _vault_collateral_value_e6(&self, value_id: u128) -> Result<Balance, VaultError>;
    fn _update_vault_debt(&mut self, vault_id: &u128) -> Result<Balance, VaultError>;
    fn _update_cuurent_interest_coefficient_e12(&mut self) -> Result<u128, VaultError>;
    fn _get_debt_by_id(&self, vault_id: &u128) -> Result<Balance, VaultError>;
    fn _get_collateral_by_id(&self, vault_id: &u128) -> Result<Balance, VaultError>;
    fn _get_last_interest_coefficient_by_id_e12(
        &self,
        vault_id: &u128,
    ) -> Result<Balance, VaultError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VaultError {
    OwnerUnexists,
    DebtUnexists,
    CollateralUnexists,
    Unexists,
    Exists,
    HasDebt,
    NotEmpty,
    VaultOwnership,
    CollateralBelowMinimum,
    CollateralAboveMinimum,
    DepositError,
    PSP34Error(PSP34Error),
    PausableError(PausableError),
    CollaterallingError(CollaterallingError),
    OwnableError(OwnableError),
    EmittingError(EmittingError),
    VEatingError(VEatingError),
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

impl From<VEatingError> for VaultError {
    fn from(error: VEatingError) -> Self {
        VaultError::VEatingError(error)
    }
}
impl From<CollaterallingError> for VaultError {
    fn from(error: CollaterallingError) -> Self {
        VaultError::CollaterallingError(error)
    }
}
