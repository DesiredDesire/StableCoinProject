use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::measuring::*;
pub use crate::traits::psp22_rated::*;
pub use crate::traits::stable_controlling::*;

const INTEREST_STEP: i128 = 318;

impl<T: SControllingStorage> SControlling for T {
    default fn control_stable_coin(&mut self) -> Result<(), SControllingError> {
        let measurer_address: AccountId = SControllingStorage::get(self).measurer_address;
        let stability_measure: u8 =
            MeasuringRef::update_stability_measure_parameter(&measurer_address)?;
        let stalbe_address: AccountId = SControllingStorage::get(self).stable_address;
        let interest_rate: i128 =
            self._stability_measure_parameter_to_interest_rate(stability_measure);
        PSP22RatedRef::be_controlled(&stalbe_address, interest_rate)?;
        Ok(())
    }
}

impl<T: SControllingStorage> SControllingInternal for T {
    default fn _stability_measure_parameter_to_interest_rate(&self, stability_measure: u8) -> i128 {
        match stability_measure {
            206..=255 => ((stability_measure - 205) as i128) * INTEREST_STEP, //Turning negative rates for holders
            50..=205 => 0,
            0..=49 => -((50 - stability_measure) as i128) * INTEREST_STEP, //Turning positive rates for holders
        }
    }
}
