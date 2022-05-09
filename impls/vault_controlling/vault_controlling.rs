pub use super::data::*;
pub use crate::traits::vault_controlling::*;

use crate::traits::system::SystemRef;
use brush::{contracts::ownable::*, modifiers};

impl<T: VControllingStorage + OwnableStorage> VControlling for T {
    default fn get_maximum_minimum_collateral_e6(&mut self) -> u128 {
        0
    }

    default fn get_vault_parameters(&mut self) -> (u128, u128, u128) {
        (
            VControllingStorage::get(self).minimum_collateral_coef_step_e6,
            VControllingStorage::get(self).interest_rate_step_e12,
            VControllingStorage::get(self).stable_coin_interest_rate_step_e12,
        )
    }

    default fn get_stability_measure_parameter(&self) -> u8 {
        self._get_stability_measure_parameter()
    }

    default fn update_stability_measure_parameter(&mut self) -> Result<u8, VControllingError> {
        self._update_stability_measure_parameter()
    }

    #[modifiers(only_owner)]
    default fn set_vault_parameters(
        &mut self,
        interest_rate_step_e12: u128,
        minimum_collateral_coef_step_e6: u128,
        stable_coin_interest_rate_step_e12: u128,
    ) -> Result<(), VControllingError> {
        VControllingStorage::get_mut(self).interest_rate_step_e12 = interest_rate_step_e12;
        VControllingStorage::get_mut(self).minimum_collateral_coef_step_e6 =
            minimum_collateral_coef_step_e6;
        VControllingStorage::get_mut(self).stable_coin_interest_rate_step_e12 =
            stable_coin_interest_rate_step_e12;
        Ok(())
    }

    default fn feed_interest_rate(&mut self) -> Result<u128, VControllingError> {
        let stability_measure_parameter = self._update_stability_measure_parameter()?;
        let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
            self._stability_measure_parameter_to_vault_parameters(stability_measure_parameter)?;
        Ok(interest_rate_e12)
    }

    default fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VControllingError> {
        let stability_measure_parameter = self._update_stability_measure_parameter()?;
        let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
            self._stability_measure_parameter_to_vault_parameters(stability_measure_parameter)?;
        Ok(interest_rate_e12)
    }

    default fn feed_all(&mut self) -> Result<(u128, u128, u128), VControllingError> {
        let stability_measure_parameter = self._update_stability_measure_parameter()?;
        Ok(self._stability_measure_parameter_to_vault_parameters(stability_measure_parameter)?)
    }
}

impl<T: VControllingStorage + OwnableStorage> VControllingInternal for T {
    default fn _update_stability_measure_parameter(&mut self) -> Result<u8, VControllingError> {
        Ok(SystemRef::update_stability_measure_parameter(
            &VControllingStorage::get(self).system_address,
        )?)
    }

    default fn _get_stability_measure_parameter(&self) -> u8 {
        //TODO
        127 as u8
    }
    default fn _stability_measure_parameter_to_vault_parameters(
        &self,
        stability_measure_parameter: u8,
    ) -> Result<(u128, u128, u128), VControllingError> {
        // (interest rate_e12, minimum_collateral_ratio, part_for_holders)
        let min_col_step: u128 = VControllingStorage::get(self).minimum_collateral_coef_step_e6;
        let interest_rate_step_e12: u128 = VControllingStorage::get(self).interest_rate_step_e12;
        let stable_coin_interest_rate_step_e12: u128 =
            VControllingStorage::get(self).stable_coin_interest_rate_step_e12;
        Ok(match stability_measure_parameter {
            206..=255 => (
                0,
                VControllingStorage::get(self).MIN_COL_E6 - 50 * min_col_step,
                0,
            ), //Turning negative rates for holders, consider adding positive rates
            156..=205 => (
                0,
                VControllingStorage::get(self).MIN_COL_E6
                    - (stability_measure_parameter as u128 - 155) * min_col_step,
                0,
            ),
            131..=155 => (
                (155 - stability_measure_parameter) as u128 * interest_rate_step_e12,
                VControllingStorage::get(self).MIN_COL_E6,
                0,
            ),
            125..=130 => (
                25 * interest_rate_step_e12,
                VControllingStorage::get(self).MIN_COL_E6,
                0,
            ),
            50..=124 => (
                (150 - stability_measure_parameter) as u128 * interest_rate_step_e12,
                VControllingStorage::get(self).MIN_COL_E6,
                0,
            ),
            0..=49 => (
                (150 - stability_measure_parameter) as u128 * interest_rate_step_e12,
                VControllingStorage::get(self).MIN_COL_E6,
                (50 - stability_measure_parameter) as u128 * stable_coin_interest_rate_step_e12,
            ),
        })
    }
}
