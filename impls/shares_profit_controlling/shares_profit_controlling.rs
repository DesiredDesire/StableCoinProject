use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::shares_profit_controlling::*;
pub use crate::traits::shares_profit_generating::*;
use brush::{contracts::ownable::*, contracts::traits::psp22::extensions::mintable::*, modifiers};
use ink_env::CallFlags;

const E6: u128 = 10_u128.pow(6);

impl<T: SPControllingStorage + OwnableStorage> SPControlling for T {
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
        let stable_coin_address: AccountId = SPControllingStorage::get(self).stable_coin_address;
        let treassuty_address: AccountId = SPControllingStorage::get(self).treassury_address;
        let owner: AccountId = OwnableStorage::get(self).owner;
        let treassury_part_e6: u128 = SPControllingStorage::get(self).treassury_part_e6;
        let treassury_profit: u128 = profit as u128 * treassury_part_e6 / E6;
        PSP22MintableRef::mint_builder(&stable_coin_address, treassuty_address, treassury_profit)
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap()?;
        PSP22MintableRef::mint_builder(
            &stable_coin_address,
            owner,
            profit as u128 - treassury_profit,
        )
        .call_flags(CallFlags::default().set_allow_reentry(true))
        .fire()
        .unwrap()?;
        SPControllingStorage::get_mut(self).minted_amount += profit as u128;
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_is_generator(
        &mut self,
        account: AccountId,
        is: bool,
    ) -> Result<(), SPControllingError> {
        SPControllingStorage::get_mut(self)
            .is_generator
            .insert(&account, &is);
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
    default fn get_stable_coin_address(&self) -> AccountId {
        SPControllingStorage::get(self).stable_coin_address.clone()
    }

    default fn is_generator(&self, account: AccountId) -> bool {
        SPControllingStorage::get(self)
            .is_generator
            .get(&account)
            .unwrap_or(false) //TODO make it internal function
    }

    default fn get_total_profit(&self) -> i128 {
        SPControllingStorage::get(self).total_profit.clone()
    }

    default fn get_treassury_address(&self) -> AccountId {
        SPControllingStorage::get(self).treassury_address.clone()
    }

    default fn get_treassury_part_e6(&self) -> u128 {
        SPControllingStorage::get(self).treassury_part_e6.clone()
    }
}
