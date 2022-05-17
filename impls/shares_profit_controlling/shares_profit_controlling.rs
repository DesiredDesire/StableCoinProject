use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::shares_profit_controlling::*;
pub use crate::traits::shares_profit_generating::*;
use brush::{contracts::ownable::*, contracts::pausable::*, modifiers};

impl<T: SPControllingStorage + OwnableStorage + PausableStorage> SPControlling for T {
    default fn collect_profit(
        &mut self,
        profit_generator: AccountId,
    ) -> Result<i128, SPControllingError> {
        if !SPControllingStorage::get(self)
            .is_generator
            .get(&profit_generator)
            .unwrap_or(false)
        {
            return Err(SPControllingError::Generator);
        }
        let collected_profit: i128 = SPGeneratingRef::give_profit(&profit_generator)?;
        SPControllingStorage::get_mut(self).total_profit += collected_profit;
        Ok(collected_profit)
    }

    default fn distribute_income(&mut self) -> Result<(), SPControllingError> {
        let profit: i128 = SPControllingStorage::get(self).total_profit;
        if profit <= 0 {
            return Err(SPControllingError::NoProfit);
        }
        SPControllingStorage::get_mut(self).total_profit = 0;

        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_treassury_address(
        &mut self,
        new_treassury_address: AccountId,
    ) -> Result<(), SPControllingError> {
        SPControllingStorage::get_mut(self).treassury_address = new_treassury_address;
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_treassury_part_e6(
        &mut self,
        new_treassury_part_e6: u128,
    ) -> Result<(), SPControllingError> {
        if new_treassury_part_e6 > 1000000 {
            return Err(SPControllingError::One);
        }
        SPControllingStorage::get_mut(self).treassury_part_e6 = new_treassury_part_e6;
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_sharing_part_e6(
        &mut self,
        profit_generator: AccountId,
        new_sharing_part_e6: u128,
    ) -> Result<(), SPControllingError> {
        if !SPControllingStorage::get(self)
            .is_generator
            .get(&profit_generator)
            .unwrap_or(false)
        {
            return Err(SPControllingError::Generator);
        }
        SPGeneratingRef::set_sharing_part_e6(&(profit_generator), new_sharing_part_e6)?;
        Ok(())
    }
}

impl<T: SPControllingStorage> SPControllingView for T {
    default fn get_total_profit(&self) -> i128 {
        SPControllingStorage::get(self).total_profit.clone()
    }

    default fn get_treassury_address(&self) -> AccountId {
        SPControllingStorage::get(self).treassury_address.clone()
    }
}
