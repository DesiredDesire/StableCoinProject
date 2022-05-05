pub use super::data::*;
pub use crate::traits::vault_eating::*;
pub use crate::traits::vault_feeding::*;

use brush::contracts::ownable::*;
use brush::modifiers;
use brush::traits::AccountId;

impl<T: VEatingStorage + OwnableStorage> VEating for T {
    default fn eat_collateral_price_e6(&self) -> Result<u128, VEatingError> {
        match VFeedingRef::feed_price(&VEatingStorage::get(self).feeder_address) {
            Ok(v) => Ok(v),
            Err(e) => Err(VEatingError::from(e)),
        }
    }
    default fn eat_interest_rate_e12(&self) -> Result<u128, VEatingError> {
        match VFeedingRef::feed_interest_rate(&VEatingStorage::get(self).feeder_address) {
            Ok(v) => Ok(v),
            Err(e) => Err(VEatingError::from(e)),
        }
    }
    default fn eat_minimum_collateral_coefficient_e6(&self) -> Result<u128, VEatingError> {
        match VFeedingRef::feed_minimum_collateral_coefficient(
            &VEatingStorage::get(self).feeder_address,
        ) {
            Ok(v) => Ok(v),
            Err(e) => Err(VEatingError::from(e)),
        }
    }

    default fn eat_all(&self) -> Result<(u128, u128, u128), VEatingError> {
        match VFeedingRef::feed_all(&VEatingStorage::get(self).feeder_address) {
            Ok(v) => Ok(v),
            Err(e) => Err(VEatingError::from(e)),
        }
    }

    #[modifiers(only_owner)]
    default fn change_feeder(&mut self, new_feeder_address: AccountId) -> Result<(), VEatingError> {
        let old_feeder = VEatingStorage::get(self).feeder_address;
        VEatingStorage::get_mut(self).feeder_address = new_feeder_address;
        self._emit_feeder_changed_event(
            Some(old_feeder),
            Some(new_feeder_address),
            Self::env().caller(),
        );
        Ok(())
    }

    default fn get_feeder_address(&self) -> AccountId {
        VEatingStorage::get(self).feeder_address
    }
}

impl<T: VEatingStorage + OwnableStorage> VEatingInternal for T {
    default fn _emit_feeder_changed_event(
        &self,
        _old_feeder: Option<AccountId>,
        _new_feeder: Option<AccountId>,
        _caller: AccountId,
    ) { //TODO
    }
}
