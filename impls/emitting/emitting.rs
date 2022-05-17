pub use super::data::*;
pub use crate::traits::emitting::*;

use brush::{
    contracts::traits::psp22::extensions::{burnable::*, mintable::*},
    traits::{AccountId, Balance},
};

impl<T: EmittingStorage> Emitting for T {
    default fn emited_amount(&self) -> Balance {
        EmittingStorage::get(self).emited_amount
    }
    default fn get_emited_token_address(&self) -> AccountId {
        EmittingStorage::get(self).emited_token_address
    }
}

impl<T: EmittingStorage> EmittingInternal for T {
    default fn _mint_emited_token(
        &mut self,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), EmittingError> {
        ink_env::debug_println!("amint mount: {}", amount);
        let emited_token_address = EmittingStorage::get(self).emited_token_address;
        EmittingStorage::get_mut(self).emited_amount += amount;
        PSP22MintableRef::mint(&emited_token_address, to, amount)?;
        Ok(())
    }

    default fn _burn_emited_token(
        &mut self,
        from: AccountId,
        amount: Balance,
    ) -> Result<(), EmittingError> {
        let emited_token_address = EmittingStorage::get(self).emited_token_address;
        PSP22BurnableRef::burn(&emited_token_address, from, amount)?;
        EmittingStorage::get_mut(self).emited_amount -= amount;
        Ok(())
    }
}
