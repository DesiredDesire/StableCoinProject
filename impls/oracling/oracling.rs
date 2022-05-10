pub use super::data::*;
pub use crate::traits::oracling::*;
pub use crate::traits::stable_coin::*;

impl<T: OraclingStorage> Oracling for T {
    fn get_azero_usd_price_e6(&self) -> u128 {
        OraclingStorage::get(self).azero_usd_price_e6
    }

    fn get_azero_ausd_price_e6(&self) -> u128 {
        OraclingStorage::get(self).azero_ausd_price_e6
    }
}
