pub use super::data::*;
pub use crate::traits::measuring::*;
pub use crate::traits::oracling::*;
pub use crate::traits::psp22_rated::*;
use brush::contracts::ownable::*;
use brush::modifiers;
use brush::traits::AccountId;
use brush::traits::Timestamp;

const E6: u128 = 1000000;
const MAXU128: u128 = 340282366920938463463374607431768211455;
const SECOND: Timestamp = 1;
const MINUTE: Timestamp = 60 * SECOND;
const HOUR: Timestamp = 60 * MINUTE;

impl<T: MeasuringStorage + OwnableStorage> Measuring for T {
    // #[brush::modifiers(when_not_paused)] // TODO think about it
    default fn update_stability_measure_parameter(&mut self) -> Result<u8, MeasuringError> {
        let oracle_address = MeasuringStorage::get(self).oracle_address;
        let azero_usd_price_e6 = OraclingRef::get_azero_usd_price_e6(&oracle_address);
        let azero_ausd_price_e6 = OraclingRef::get_azero_ausd_price_e6(&oracle_address);
        let ausd_usd_price_e6 = azero_usd_price_e6 * E6 / azero_ausd_price_e6;
        MeasuringStorage::get_mut(self).ausd_usd_price_e6 = ausd_usd_price_e6;
        let last_measurement_timestamp = MeasuringStorage::get(self).measurement_timestamp;
        let current_timestamp = Self::env().block_timestamp();
        let time_passed = current_timestamp - last_measurement_timestamp;

        if _ausd_usd_price_e6_to_measurement_period(ausd_usd_price_e6) < time_passed {
            if azero_ausd_price_e6 > 1001000 {
                MeasuringStorage::get_mut(self).stability_measure += 1;
            } else if azero_ausd_price_e6 < 999000 {
                MeasuringStorage::get_mut(self).stability_measure -= 1;
            } else {
                if MeasuringStorage::get(self).stability_measure > 128 {
                    MeasuringStorage::get_mut(self).stability_measure -= 1;
                } else if MeasuringStorage::get(self).stability_measure < 128 {
                    MeasuringStorage::get_mut(self).stability_measure += 1;
                }
            }
            MeasuringStorage::get_mut(self).measurement_timestamp = current_timestamp;
        }
        Ok(MeasuringStorage::get(self).stability_measure)
    }

    #[modifiers(only_owner)]
    fn set_oracle_address(&mut self, new_oracle_address: AccountId) -> Result<(), MeasuringError> {
        MeasuringStorage::get_mut(self).oracle_address = new_oracle_address;
        Ok(())
    }
}

impl<T: MeasuringStorage> MeasuringView for T {
    default fn get_stability_measure_parameter(&self) -> u8 {
        MeasuringStorage::get(self).stability_measure
    }

    default fn get_ausd_usd_price_e6(&self) -> u128 {
        MeasuringStorage::get(self).ausd_usd_price_e6
    }

    default fn get_measurement_timestamp(&self) -> Timestamp {
        MeasuringStorage::get(self).measurement_timestamp
    }

    fn get_oracle_address(&self) -> AccountId {
        MeasuringStorage::get(self).oracle_address
    }
}

fn _ausd_usd_price_e6_to_measurement_period(ausd_usd_price_e6: u128) -> Timestamp {
    match ausd_usd_price_e6 {
        0..=940000 => 30 * SECOND,
        950001..=960000 => 1 * MINUTE,
        960001..=970000 => 5 * MINUTE,
        970001..=980000 => 15 * MINUTE,
        980001..=990000 => 30 * MINUTE,
        990001..=995000 => HOUR,
        995001..=999000 => 2 * HOUR,
        999001..=1001000 => 2 * HOUR,
        100101..=1005000 => 2 * HOUR,
        100501..=1010000 => HOUR,
        101001..=1020000 => 30 * MINUTE,
        102001..=1030000 => 15 * MINUTE,
        103001..=1040000 => 5 * MINUTE,
        104001..=1050000 => 1 * MINUTE,
        105001..=MAXU128 => 30 * SECOND,
    }
}
