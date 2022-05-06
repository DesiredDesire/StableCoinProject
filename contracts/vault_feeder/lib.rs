#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use brush::modifiers;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::vault_feeding::*;

    const MIN_COL_E6: u128 = 2 * 10 ^ 6;
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, VFeedingStorage)]
    pub struct VFeederContract {
        #[OwnableStorageField]
        owner: OwnableData,
        #[VFeedingStorageField]
        feed: VFeedingData,
    }

    impl Ownable for VFeederContract {}

    impl VFeeding for VFeederContract {
        #[ink(message)]
        fn get_maximum_minimum_collateral_e6(&mut self) -> u128 {
            MIN_COL_E6
        }

        #[ink(message)]
        fn get_vault_parameters(&mut self) -> (u128, u128, u128) {
            (
                self.feed.interest_rate_step_e12,
                self.feed.minimum_collateral_coef_step_e6,
                self.feed.stable_coin_interest_rate_step_e12,
            )
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        fn set_vault_parameters(
            &mut self,
            interest_rate_step_e12: u128,
            minimum_collateral_coef_step_e6: u128,
            stable_coin_interest_rate_step_e12: u128,
        ) -> Result<(), VFeedingError> {
            self.feed.interest_rate_step_e12 = interest_rate_step_e12;
            self.feed.minimum_collateral_coef_step_e6 = minimum_collateral_coef_step_e6;
            self.feed.stable_coin_interest_rate_step_e12 = stable_coin_interest_rate_step_e12;
            Ok(())
        }

        #[ink(message)]
        fn get_state_parameter(&mut self) -> Result<u8, VFeedingError> {
            self._get_state_parameter()
        }

        #[ink(message)]
        fn feed_interest_rate(&mut self) -> Result<u128, VFeedingError> {
            let state_parameter = self._get_state_parameter()?;
            let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
                self._state_parameter_to_vault_parameters(state_parameter)?;
            Ok(interest_rate_e12)
        }

        #[ink(message)]
        fn feed_minimum_collateral_coefficient(&mut self) -> Result<u128, VFeedingError> {
            let state_parameter = self._get_state_parameter()?;
            let (interest_rate_e12, _minimum_collateral_ratio_e6, _part_for_holder_e6) =
                self._state_parameter_to_vault_parameters(state_parameter)?;
            Ok(interest_rate_e12)
        }

        #[ink(message)]
        fn feed_all(&mut self) -> Result<(u128, u128, u128), VFeedingError> {
            let state_parameter = self._get_state_parameter()?;
            Ok(self._state_parameter_to_vault_parameters(state_parameter)?)
        }
    }

    impl VFeedingInternal for VFeederContract {
        fn _get_state_parameter(&self) -> Result<u8, VFeedingError> {
            //TODO
            Ok(127)
        }
        fn _state_parameter_to_vault_parameters(
            &self,
            state_parameter: u8,
        ) -> Result<(u128, u128, u128), VFeedingError> {
            // (interest rate_e12, minimum_collateral_ratio, part_for_holders)
            let min_col_step_e6: u128 = self.feed.minimum_collateral_coef_step_e6;
            let interest_rate_step_e12: u128 = self.feed.interest_rate_step_e12;
            let stable_coin_interest_rate_step_e12: u128 =
                self.feed.stable_coin_interest_rate_step_e12;
            Ok(match state_parameter {
                206..=255 => (0, MIN_COL_E6 - 50 * min_col_step_e6, 0), //Turning negative rates for holders, consider adding positive rates
                156..=205 => (
                    0,
                    MIN_COL_E6 - (state_parameter as u128 - 155) * min_col_step_e6,
                    0,
                ),
                131..=155 => (
                    (155 - state_parameter) as u128 * interest_rate_step_e12,
                    MIN_COL_E6,
                    0,
                ),
                125..=130 => (25 * interest_rate_step_e12, MIN_COL_E6, 0),
                50..=124 => (
                    (150 - state_parameter) as u128 * interest_rate_step_e12,
                    MIN_COL_E6,
                    0,
                ),
                0..=49 => (
                    (150 - state_parameter) as u128 * interest_rate_step_e12,
                    MIN_COL_E6,
                    (50 - state_parameter) as u128 * stable_coin_interest_rate_step_e12,
                ),
            })
        }
    }

    impl VFeederContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(
            protocol_state_address: AccountId,
            interest_rate_step_e12: u128,
            minimum_collateral_coef_step_e6: u128,
            stable_coin_interest_rate_step_e12: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VFeederContract| {
                instance.feed.protocol_state_address = protocol_state_address;
                instance.feed.interest_rate_step_e12 = interest_rate_step_e12;
                instance.feed.minimum_collateral_coef_step_e6 = minimum_collateral_coef_step_e6;
                instance.feed.stable_coin_interest_rate_step_e12 =
                    stable_coin_interest_rate_step_e12;
            })
        }
    }
}
