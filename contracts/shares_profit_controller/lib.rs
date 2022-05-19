#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::shares_profit_controlling::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, SPControllingStorage)]
    pub struct SPControllerContract {
        #[OwnableStorageField]
        owner: OwnableData,
        #[SPControllingStorageField]
        control: SPControllingData,
    }

    impl Ownable for SPControllerContract {}

    impl SPControlling for SPControllerContract {}

    impl SPControllingView for SPControllerContract {}

    impl SPControllingInternal for SPControllerContract {}

    impl SPControllerContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(stable_coin_address: AccountId, owner: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut SPControllerContract| {
                instance.control.stable_coin_address = stable_coin_address;
                instance._init_with_owner(owner);
            })
        }
    }
}
