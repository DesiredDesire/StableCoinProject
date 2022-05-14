#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::oracling::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, OraclingStorage)]
    pub struct OracleContract {
        #[OwnableStorageField]
        owner: OwnableData,
        #[OraclingStorageField]
        oracle: OraclingData,
    }

    impl Ownable for OracleContract {}

    impl Oracling for OracleContract {}

    impl OracleContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut OracleContract| {
                instance.oracle.azero_usd_price_e6 = 0;
                instance.oracle.azero_ausd_price_e6 = 0;
                instance._init_with_owner(owner);
            })
        }

        #[ink(message)]
        pub fn feed_azero_usd_price_e6(&mut self, azero_usd_price_e6: u128) {
            self.oracle.azero_usd_price_e6 = azero_usd_price_e6;
        }

        #[ink(message)]
        pub fn feed_azero_ausd_price_e6(&mut self, azero_ausd_price_e6: u128) {
            self.oracle.azero_ausd_price_e6 = azero_ausd_price_e6;
        }
    }
}
