pub use super::data::*;
pub use crate::traits::oracling::*;

impl<T: OraclingStorage> Oracling for T {
    fn get_azero_usd_price_e6(&self) -> u128 {
        OraclingStorage::get(self).azero_usd_price_e6
    }

    fn get_azero_ausd_price_e6(&self) -> u128 {
        OraclingStorage::get(self).azero_ausd_price_e6
    }

    // TODO add new funciton that returns ratio
}
