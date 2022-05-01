pub use super::data::*;
pub use crate::traits::eating::*;
pub use crate::traits::feeding::*;

use brush::contracts::ownable::*;
use brush::traits::AccountId;

impl<T: EatingStorage + OwnableStorage> Eating for T {
    default fn eat_price(&self) -> Result<u128, EatingError> {
        match FeedingRef::feed_price(&EatingStorage::get(self).feeder_address) {
            Ok(v) => Ok(v),
            Err(e) => Err(EatingError::from(e)),
        }
    }

    default fn change_feeder(&mut self, new_feeder_address: AccountId) -> Result<(), EatingError> {
        let old_feeder = EatingStorage::get(self).feeder_address;
        EatingStorage::get_mut(self).feeder_address = new_feeder_address;
        self._emit_feeder_changed_event(
            Some(old_feeder),
            Some(new_feeder_address),
            Self::env().caller(),
        );
        Ok(())
    }
}

impl<T: EatingStorage + OwnableStorage> EatingInternal for T {
    default fn _emit_feeder_changed_event(
        &self,
        _old_feeder: Option<AccountId>,
        _new_feeder: Option<AccountId>,
        _caller: AccountId,
    ) {
    }
}
