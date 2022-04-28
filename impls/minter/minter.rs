pub use super::data::*;
pub use crate::traits::minter::*;
pub use crate::traits::stable_coin::*;
use brush::{
    contracts::pausable::*,
    modifiers,
    traits::{AccountId, Balance},
};

impl<T: MinterStorage + PausableStorage> Minter for T {
    #[modifiers(when_not_paused)]
    default fn mint(&mut self, to: AccountId, amount: Balance) -> Result<(), MinterError> {
        let minted_token_address = MinterStorage::get(self).minted_token_address;
        match StableCoinContractRef::mint(&minted_token_address, to, amount) {
            Ok(r) => Ok(r),
            Err(e) => Err(MinterError::from(e)),
        }
    }
}
