pub use super::data::*;
pub use crate::traits::vault_feeding::*;

use brush::{contracts::ownable::*, modifiers};

const MIN_COL: u128 = 2 * 10 ^ 6;

impl<T: VFeedingStorage + OwnableStorage> VFeeding for T {
    default fn get_maximum_minimum_collateral_e6(&mut self) -> u128 {
        0
    }

    default fn get_vault_parameters(&mut self) -> (u128, u128, u128) {
        (
            VFeedingStorage::get(self).minimum_collateral_coef_step_e6,
            VFeedingStorage::get(self).interest_rate_step_e12,
            VFeedingStorage::get(self).stable_coin_interest_rate_step_e12,
        )
    }

    default fn get_state_parameter(&mut self) -> Result<u8, VFeedingError> {
        self._get_state_parameter()
    }

    #[modifiers(only_owner)]
    default fn set_vault_parameters(
        &mut self,
        interest_rate_step_e12: u128,
        minimum_collateral_coef_step_e6: u128,
        stable_coin_interest_rate_step_e12: u128,
    ) -> Result<(), VFeedingError> {
        VFeedingStorage::get_mut(self).interest_rate_step_e12 = interest_rate_step_e12;
        VFeedingStorage::get_mut(self).minimum_collateral_coef_step_e6 =
            minimum_collateral_coef_step_e6;
        VFeedingStorage::get_mut(self).stable_coin_interest_rate_step_e12 =
            stable_coin_interest_rate_step_e12;
        Ok(())
    }

    default fn feed_interest_rate(&mut self) -> Result<u128, VFeedingError> {
        let state_parameter = self._get_state_parameter()?;
        let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
            self._state_parameter_to_vault_parameters(state_parameter)?;
        Ok(interest_rate_e12)
    }

    default fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VFeedingError> {
        let state_parameter = self._get_state_parameter()?;
        let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
            self._state_parameter_to_vault_parameters(state_parameter)?;
        Ok(interest_rate_e12)
    }

    default fn feed_all(&mut self) -> Result<(u128, u128, u128), VFeedingError> {
        let state_parameter = self._get_state_parameter()?;
        Ok(self._state_parameter_to_vault_parameters(state_parameter)?)
    }
}

impl<T: VFeedingStorage + OwnableStorage> VFeedingInternal for T {
    default fn _get_state_parameter(&self) -> Result<u8, VFeedingError> {
        //TODO
        Ok(127)
    }
    default fn _state_parameter_to_vault_parameters(
        &self,
        state_parameter: u8,
    ) -> Result<(u128, u128, u128), VFeedingError> {
        // (interest rate_e12, minimum_collateral_ratio, part_for_holders)
        let min_col_step: u128 = VFeedingStorage::get(self).minimum_collateral_coef_step_e6;
        let interest_rate_step_e12: u128 = VFeedingStorage::get(self).interest_rate_step_e12;
        let stable_coin_interest_rate_step_e12: u128 =
            VFeedingStorage::get(self).stable_coin_interest_rate_step_e12;
        Ok(match state_parameter {
            206..=255 => (0, MIN_COL - 50 * min_col_step, 0), //Turning negative rates for holders, consider adding positive rates
            156..=205 => (
                0,
                MIN_COL - (state_parameter as u128 - 155) * min_col_step,
                0,
            ),
            131..=155 => (
                (155 - state_parameter) as u128 * interest_rate_step_e12,
                MIN_COL,
                0,
            ),
            125..=130 => (25 * interest_rate_step_e12, MIN_COL, 0),
            50..=124 => (
                (150 - state_parameter) as u128 * interest_rate_step_e12,
                MIN_COL,
                0,
            ),
            0..=49 => (
                (150 - state_parameter) as u128 * interest_rate_step_e12,
                MIN_COL,
                (50 - state_parameter) as u128 * stable_coin_interest_rate_step_e12,
            ),
        })
    }
}
