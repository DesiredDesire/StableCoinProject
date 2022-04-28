pub use super::data::*;
pub use crate::traits::stable_coin::*;
pub use crate::traits::vault::*;
use brush::contracts::psp34::Id;
use brush::contracts::psp34::PSP34Storage;
use brush::{
    contracts::pausable::*,
    modifiers,
    traits::{AccountId, Balance},
};

impl<T: VaultStorage + PSP34Storage> Vault for T {
    default fn deposit(&mut self, amount: Balance) -> Result<(), VaultError> {
        Ok(())
    }

    default fn withdraw(&mut self, amount: Balance) -> Result<(), VaultError> {
        Ok(())
    }

    default fn get_debt_ceiling(&mut self) -> Balance {
        1
    }

    default fn create_vault(&mut self) -> Result<Id, VaultError> {
        Ok(Id::U8(1))
    }

    default fn destroy_vault(&mut self, vault_id: Id) -> Result<(), VaultError> {
        Ok(())
    }

    default fn transfer_vault(&mut self, vault_id: Id, address_to: Id) -> () {}

    default fn deposit_collateral(&mut self, vault_id: Id, amount: Balance) -> () {}

    default fn withdraw_collateral(&mut self, vault_id: Id, amount: Balance) -> () {}

    default fn borrow_token(&mut self, vault_id: Id, amount: Balance) -> () {}

    default fn pay_back_token(&mut self, vault_id: Id, amount: Balance) -> () {}

    default fn buy_risky_vault(&mut self, vault_id: Id) -> () {}
}
