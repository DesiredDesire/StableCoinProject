#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::vault_controlling::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, VControllingStorage)]
    pub struct VControllerContract {
        #[OwnableStorageField]
        owner: OwnableData,
        #[VControllingStorageField]
        feed: VControllingData,
    }

    impl Ownable for VControllerContract {}

    impl VControlling for VControllerContract {}

    impl VControllingInternal for VControllerContract {}

    impl VControllerContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(
            system_address: AccountId,
            interest_rate_step_e12: u128,
            minimum_collateral_coef_step_e6: u128,
            stable_coin_interest_rate_step_e12: u128,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VControllerContract| {
                instance.feed.system_address = system_address;
                instance.feed.interest_rate_step_e12 = interest_rate_step_e12;
                instance.feed.minimum_collateral_coef_step_e6 = minimum_collateral_coef_step_e6;
                instance.feed.stable_coin_interest_rate_step_e12 =
                    stable_coin_interest_rate_step_e12;
            })
        }
    }
}
