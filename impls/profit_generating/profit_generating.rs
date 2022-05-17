use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::profit_controlling::*;
use brush::{contracts::ownable::*, modifiers};

impl<T: PGeneratingStorage + OwnableStorage> PGenerating for T {
    fn give_profit(&mut self) -> Result<i128, PGeneratingError> {
        if Self::env().caller() != PGeneratingStorage::get(self).profit_controller {
            return Err(PGeneratingError::Controller);
        }
        let income: i128 = PGeneratingStorage::get(self).generated_income;
        PGeneratingStorage::get_mut(self).generated_income = 0;
        Ok(income)
    }

    #[modifiers(only_owner)]
    fn set_profit_controller_address(
        &mut self,
        new_profit_controller: AccountId,
    ) -> Result<(), PGeneratingError> {
        PGeneratingStorage::get_mut(self).profit_controller = new_profit_controller;
        Ok(())
    }
}

impl<T: PGeneratingStorage> PGeneratingInternal for T {
    fn _add_income(&mut self, amount: u128) {
        PGeneratingStorage::get_mut(self).generated_income += amount as i128;
    }
    fn _sub_income(&mut self, amount: u128) {
        PGeneratingStorage::get_mut(self).generated_income -= (amount as i128);
    }
}
