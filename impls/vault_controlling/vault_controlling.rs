pub use super::data::*;
pub use crate::traits::measuring::*;
pub use crate::traits::vault::*;
pub use crate::traits::vault_controlling::*;
use brush::traits::AccountId;

impl<T: VControllingStorage> VControlling for T {
    default fn control_vault(&mut self) -> Result<(), VControllingError> {
        let measurer_address = VControllingStorage::get(self).measurer_address;
        let stability_measure =
            MeasuringRef::update_stability_measure_parameter(&measurer_address)?;
        let vault_address = VControllingStorage::get(self).vault_address;
        let (interest_rate_step, collateral_step, stable_coin_interest_rate_step) =
            self._stability_measure_parameter_to_vault_parameters(stability_measure);
        VaultRef::be_controlled(
            &vault_address,
            interest_rate_step,
            collateral_step,
            stable_coin_interest_rate_step,
        )?;

        Ok(())
    }
}

impl<T: VControllingStorage> VControllingView for T {
    default fn get_vault_address(&self) -> AccountId {
        VControllingStorage::get(self).vault_address
    }

    default fn get_measurer_address(&self) -> AccountId {
        VControllingStorage::get(self).measurer_address
    }
}

impl<T: VControllingStorage> VControllingInternal for T {
    default fn _stability_measure_parameter_to_vault_parameters(
        &self,
        stability_measure: u8,
    ) -> (i16, u16, i16) {
        match stability_measure {
            206..=255 => (
                -((255 - 205) as i16),
                50,
                stability_measure as i16 - 205 + 55,
            ), //Turning negative rates for holders, consider adding positive rates
            156..=205 => (0, (stability_measure - 155) as u16, 0),
            131..=155 => ((155 - stability_measure) as i16, 0, 0),
            125..=130 => (25, 0, 0),
            50..=124 => ((150 - stability_measure) as i16, 0, 0),
            0..=49 => (
                (150 - stability_measure) as i16,
                0,
                stability_measure as i16 - 50,
            ),
        }
    }
}
