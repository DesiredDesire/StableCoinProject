pub use super::data::*;
pub use crate::traits::emiting::*;

use brush::{
    contracts::pausable::*,
    contracts::traits::psp22::extensions::{burnable::*, mintable::*},
    modifiers,
    traits::{AccountId, Balance},
};

impl<T: EmitingStorage + PausableStorage> Emiting for T {
    #[modifiers(when_not_paused)]
    default fn mint(&mut self, to: AccountId, amount: Balance) -> Result<(), EmitingError> {
        let emited_token_address = EmitingStorage::get(self).emited_token_address;
        match PSP22MintableRef::mint(&emited_token_address, to, amount) {
            Ok(r) => Ok(r),
            Err(e) => Err(EmitingError::from(e)),
        }
    }

    #[modifiers(when_not_paused)]
    default fn burn(&mut self, from: AccountId, amount: Balance) -> Result<(), EmitingError> {
        let emited_token_address = EmitingStorage::get(self).emited_token_address;
        match PSP22BurnableRef::burn(&emited_token_address, from, amount) {
            Ok(r) => Ok(r),
            Err(e) => Err(EmitingError::from(e)),
        }
    }
}
