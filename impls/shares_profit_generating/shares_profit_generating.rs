pub use super::data::*;
pub use crate::traits::shares_profit_generating::*;
use brush::{
    contracts::ownable::*,
    contracts::traits::psp22::extensions::mintable::*,
    modifiers,
    traits::{AccountId, Balance},
};

const E6: u128 = 10_u128.pow(6); //10^**6

impl<T: SPGeneratingStorage + OwnableStorage> SPGenerating for T {
    // profiting and shares
    default fn set_sharing_part_e6(
        &mut self,
        new_sharing_part_e6: u128,
    ) -> Result<(), SPGeneratingError> {
        if Self::env().caller() != SPGeneratingStorage::get(self).shares_profit_controller_address {
            return Err(SPGeneratingError::Controller);
        }
        SPGeneratingStorage::get_mut(self).sharing_part_e6 = new_sharing_part_e6;
        Ok(())
    }

    // profiting
    default fn give_profit(&mut self) -> Result<i128, SPGeneratingError> {
        if Self::env().caller() != SPGeneratingStorage::get(self).shares_profit_controller_address {
            return Err(SPGeneratingError::Controller);
        }
        let income: i128 = SPGeneratingStorage::get(self).generated_profit;
        SPGeneratingStorage::get_mut(self).generated_profit = 0;
        Ok(income)
    }

    #[modifiers(only_owner)]
    default fn set_shares_profit_controller_address(
        &mut self,
        new_profit_controller: AccountId,
    ) -> Result<(), SPGeneratingError> {
        SPGeneratingStorage::get_mut(self).shares_profit_controller_address = new_profit_controller;
        Ok(())
    }

    // shares
    default fn mint_shares(&mut self) -> Result<(), SPGeneratingError> {
        let caller = Self::env().caller();
        self._mint_shares(caller)?;
        Ok(())
    }
}

impl<T: SPGeneratingStorage> SPGeneratingView for T {
    // profiting and shares
    default fn get_sharing_part_e6(&self) -> u128 {
        SPGeneratingStorage::get(self).sharing_part_e6.clone()
    }
    // profiting
    default fn get_generated_profit(&self) -> i128 {
        SPGeneratingStorage::get(self).generated_profit.clone()
    }

    default fn get_shares_profit_controller_address(&self) -> AccountId {
        SPGeneratingStorage::get(self)
            .shares_profit_controller_address
            .clone()
    }

    // shares

    default fn get_shares_minting_allowance(&self, account: AccountId) -> u128 {
        self._get_shares_minting_allowance(account).clone()
    }

    default fn get_shares_token_address(&self) -> AccountId {
        SPGeneratingStorage::get(self).shares_token_address.clone()
    }
}

impl<T: SPGeneratingStorage> SPGeneratingInternal for T {
    // profiting and shares
    fn _add_profit_and_increase_shares_minting_allowance(
        &mut self,
        amount: u128,
        account: AccountId,
    ) {
        self._add_profit(amount);
        let sharing_part_e6: u128 = SPGeneratingStorage::get(self).sharing_part_e6;
        let increase_allowance_by: u128 = amount * sharing_part_e6 / E6;
        self._increase_shares_minting_allowance(account, increase_allowance_by);
    }

    // profiting
    fn _add_profit(&mut self, amount: u128) {
        SPGeneratingStorage::get_mut(self).generated_profit += amount as i128;
    }
    fn _sub_profit(&mut self, amount: u128) {
        SPGeneratingStorage::get_mut(self).generated_profit -= amount as i128;
    }

    // shares
    default fn _mint_shares(&mut self, to: AccountId) -> Result<(), SPGeneratingError> {
        let shares_token_address = SPGeneratingStorage::get(self).shares_token_address;
        let amount = self._get_shares_minting_allowance(to);
        SPGeneratingStorage::get_mut(self)
            .shares_minting_allowance
            .insert(&to, &0);
        PSP22MintableRef::mint(&shares_token_address, to, amount)?;
        Ok(())
    }
    fn _get_shares_minting_allowance(&self, account: AccountId) -> u128 {
        SPGeneratingStorage::get(self)
            .shares_minting_allowance
            .get(&account)
            .unwrap_or(0)
    }
    fn _increase_shares_minting_allowance(&mut self, account: AccountId, amount: Balance) {
        let allowance = self._get_shares_minting_allowance(account);
        SPGeneratingStorage::get_mut(self)
            .shares_minting_allowance
            .insert(&account, &(allowance + amount));
    }
}
