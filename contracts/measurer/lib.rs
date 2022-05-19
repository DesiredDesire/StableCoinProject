#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::ownable::*;
    use brush::contracts::pausable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::measuring::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, OwnableStorage, MeasuringStorage)]
    pub struct MeasurerContract {
        #[OwnableStorageField]
        owner: OwnableData,
        // #[PausableStorageField]
        // pause: PausableData,
        #[MeasuringStorageField]
        measure: MeasuringData,
    }

    impl Ownable for MeasurerContract {}

    impl Measuring for MeasurerContract {}
    impl MeasuringView for MeasurerContract {}

    impl MeasurerContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(oracle_address: AccountId, owner: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut MeasurerContract| {
                instance.measure.oracle_address = oracle_address;
                instance._init_with_owner(owner);
            })
        }
    }
}
