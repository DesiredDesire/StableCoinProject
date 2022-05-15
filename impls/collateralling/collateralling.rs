pub use super::data::*;
pub use crate::traits::collateralling::*;
use ink_env::CallFlags;
use ink_prelude::vec::Vec;

use brush::{
    contracts::{ownable::*, traits::psp22::*},
    modifiers,
    traits::{AccountId, Balance},
};

impl<T: CollaterallingStorage + OwnableStorage> Collateralling for T {
    default fn collateral_amount(&self) -> Balance {
        CollaterallingStorage::get(self).collateral_amount
    }

    #[modifiers(only_owner)]
    default fn rescue_psp22(
        &mut self,
        psp22_address: AccountId,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), CollaterallingError> {
        if psp22_address != CollaterallingStorage::get(self).collateral_token_address {
            PSP22Ref::transfer(&psp22_address, to, amount, Vec::<u8>::new())?;
        } else {
            if CollaterallingStorage::get(self).collateral_amount
                <= PSP22Ref::balance_of(&psp22_address, Self::env().account_id()) - amount
            {
                PSP22Ref::transfer(&psp22_address, to, amount, Vec::<u8>::new())?;
            }
        }
        Ok(())
    }

    default fn get_collateral_token_address(&self) -> AccountId {
        CollaterallingStorage::get(self).collateral_token_address
    }
}

impl<T: CollaterallingStorage> CollaterallingInternal for T {
    default fn _transfer_collateral_in(
        &mut self,
        from: AccountId,
        amount: Balance,
    ) -> Result<(), PSP22Error> {
        let collateral_token_address: AccountId =
            CollaterallingStorage::get(self).collateral_token_address;
        CollaterallingStorage::get_mut(self).collateral_amount += amount;

        PSP22Ref::transfer_from_builder(
            &collateral_token_address,
            from,
            Self::env().account_id(),
            amount,
            Vec::<u8>::new(),
        )
        .call_flags(CallFlags::default().set_allow_reentry(true))
        .fire()
        .unwrap()?;
        Ok(())
    }

    default fn _transfer_collateral_out(
        &mut self,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), PSP22Error> {
        ink_env::debug_println!("collaterlling_start");
        let collateral_token_address: AccountId =
            CollaterallingStorage::get(self).collateral_token_address;
        ink_env::debug_println!("collaterlling_transfer_build");
        PSP22Ref::transfer_builder(&collateral_token_address, to, amount, Vec::<u8>::new())
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap()?;
        ink_env::debug_println!("collaterlling_after_transfer_build");
        CollaterallingStorage::get_mut(self).collateral_amount -= amount;
        Ok(())
    }
}
