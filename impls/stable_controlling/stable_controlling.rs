use brush::contracts::ownable::*;
use brush::modifiers;
use brush::traits::AccountId;

pub use super::data::*;
pub use crate::traits::measuring::*;
pub use crate::traits::psp22_rated::*;
pub use crate::traits::stable_controlling::*;

const INTEREST_STEP: i128 = 318;

const E6: u128 = 10_u128.pow(6);

impl<T: SControllingStorage + OwnableStorage> SControlling for T {
    default fn control_stable_coin(&mut self) -> Result<(), SControllingError> {
        let measurer_address: AccountId = SControllingStorage::get(self).measurer_address;
        let stability_measure: u8 =
            MeasuringRef::update_stability_measure_parameter(&measurer_address)?; // TODO make it one call
        let ausd_usd_price_e6: u128 = MeasuringRef::get_ausd_usd_price_e6(&measurer_address); //TODO make it one call
        let stalbe_address: AccountId = SControllingStorage::get(self).stable_coin_address;
        let interest_rate: i128 =
            self._stability_measure_parameter_to_interest_rate(stability_measure);
        let tax_e6 = self._ausd_usd_price_e6_to_tax_e6(ausd_usd_price_e6);
        PSP22RatedRef::be_controlled(&stalbe_address, interest_rate, tax_e6)?;
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn set_measurer_address(
        &mut self,
        new_measurer_address: AccountId,
    ) -> Result<(), SControllingError> {
        SControllingStorage::get_mut(self).measurer_address = new_measurer_address;
        Ok(())
    }
}

impl<T: SControllingStorage> SControllingView for T {
    default fn get_stable_coin_address(&mut self) -> AccountId {
        SControllingStorage::get(self).stable_coin_address
    }

    default fn get_measurer_address(&mut self) -> AccountId {
        SControllingStorage::get(self).measurer_address
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

    default fn _ausd_usd_price_e6_to_tax_e6(&self, ausd_usd_price_e6: u128) -> u128 {
        match ausd_usd_price_e6 {
            0..=1005000 => 0,
            _ => (ausd_usd_price_e6 - 1005000) * E6 / ausd_usd_price_e6,
        }
    }
}
