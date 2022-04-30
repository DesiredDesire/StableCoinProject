pub use super::data::*;
pub use crate::traits::eating::*;
pub use crate::traits::feeding::*;

use brush::contracts::ownable::*;
use brush::traits::AccountId;

impl<T: EatingStorage + OwnableStorage> Eating for T {
    fn eat_price(&self) -> Result<u128, EatingError> {
        match FeedingRef::feed_price(&EatingStorage::get(self).feeder_address) {
            Ok(v) => Ok(v),
            Err(e) => Err(EatingError::from(e)),
        }
    }

    fn change_feeder(&mut self, new_feeder_address: AccountId) -> Result<(), EatingError> {
        EatingStorage::get_mut(self).feeder_address = new_feeder_address;
        Ok(())
    }
}
