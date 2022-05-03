pub use super::data::*;
pub use crate::traits::emiting::*;

use brush::{
    contracts::pausable::*,
    contracts::traits::psp22::extensions::{burnable::*, mintable::*},
    modifiers,
    traits::{AccountId, Balance},
};

impl<T: EmitingStorage> Emiting for T {
    default fn emited_amount(&self) -> Balance {
        EmitingStorage::get(self).emited_amount
    }
}

pub trait EmitingInternal {
    fn _mint_emited_token(&mut self, to: AccountId, amount: Balance) -> Result<(), EmitingError>;
    fn _burn_emited_token(&mut self, from: AccountId, amount: Balance) -> Result<(), EmitingError>;
}

impl<T: EmitingStorage> EmitingInternal for T {
    default fn _mint_emited_token(
        &mut self,
        to: AccountId,
        amount: Balance,
    ) -> Result<(), EmitingError> {
        let emited_token_address = EmitingStorage::get(self).emited_token_address;
        PSP22MintableRef::mint(&emited_token_address, to, amount)?;
        EmitingStorage::get_mut(self).emited_amount += amount;
        Ok(())
    }

    default fn _burn_emited_token(
        &mut self,
        from: AccountId,
        amount: Balance,
    ) -> Result<(), EmitingError> {
        let emited_token_address = EmitingStorage::get(self).emited_token_address;
        match PSP22BurnableRef::burn(&emited_token_address, from, amount) {
            Ok(r) => (),
            Err(e) => {
                return Err(EmitingError::from(e));
            }
        }
        EmitingStorage::get_mut(self).emited_amount += amount;
        Ok(())
    }
}
