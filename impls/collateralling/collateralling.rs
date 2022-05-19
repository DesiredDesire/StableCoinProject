pub use super::data::*;
pub use crate::traits::collateralling::*;
use ink_env::CallFlags;
use ink_prelude::vec::Vec;

use brush::{
    contracts::traits::psp22::*,
    traits::{AccountId, Balance},
};

impl<T: CollaterallingStorage> Collateralling for T {
    default fn collateral_amount(&self) -> Balance {
        CollaterallingStorage::get(self).collateral_amount
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
