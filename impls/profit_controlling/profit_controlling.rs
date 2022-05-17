use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::profit_controlling::*;
pub use crate::traits::profit_generating::*;
use brush::{contracts::ownable::*, contracts::pausable::*, contracts::psp22::*, modifiers};

impl<T: PControllingStorage + OwnableStorage + PausableStorage> PControlling for T {
    default fn collect_profit(
        &mut self,
        profit_generator: AccountId,
    ) -> Result<i128, PControllingError> {
        if !PControllingStorage::get(self)
            .is_generator
            .get(&profit_generator)
            .unwrap_or(false)
        {
            return Err(PControllingError::Generator);
        }
        let collected_profit: i128 = PGeneratingRef::give_profit(&profit_generator)?;
        PControllingStorage::get_mut(self).total_profit += collected_profit;
        Ok(collected_profit)
    }

    default fn distribute_income(&mut self) -> Result<(), PControllingError> {
        let profit: i128 = PControllingStorage::get(self).total_profit;
        if profit <= 0 {
            return Err(PControllingError::NoProfit);
        }
        PControllingStorage::get_mut(self).total_profit = 0;

        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_treassury_address(
        &mut self,
        new_treassury_address: AccountId,
    ) -> Result<(), PControllingError> {
        PControllingStorage::get_mut(self).treassury_address = new_treassury_address;
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_treassury_part_e6(
        &mut self,
        new_treassury_part_e6: u128,
    ) -> Result<(), PControllingError> {
        if new_treassury_part_e6 > 1000000 {
            return Err(PControllingError::One);
        }
        PControllingStorage::get_mut(self).treassury_part_e6 = new_treassury_part_e6;
        Ok(())
    }
}

impl<T: PControllingStorage> PControllingView for T {
    default fn get_total_profit(&self) -> i128 {
        PControllingStorage::get(self).total_profit.clone()
    }

    default fn get_treassury_address(&self) -> AccountId {
        PControllingStorage::get(self).treassury_address.clone()
    }
}
