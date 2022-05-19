#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::stable_controlling::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, SControllingStorage)]
    pub struct SControllerContract {
        #[OwnableStorageField]
        owner: OwnableData,
        #[SControllingStorageField]
        control: SControllingData,
    }

    impl Ownable for SControllerContract {}

    impl SControlling for SControllerContract {}

    impl SControllingView for SControllerContract {}

    impl SControllingInternal for SControllerContract {}

    impl SControllerContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(
            measurer_address: AccountId,
            stable_coin_address: AccountId,
            owner: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut SControllerContract| {
                instance.control.measurer_address = measurer_address;
                instance.control.stable_coin_address = stable_coin_address;
                instance._init_with_owner(owner);
            })
        }
    }
}
