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
        control: VControllingData,
    }

    impl Ownable for VControllerContract {}

    impl VControlling for VControllerContract {}

    impl VControllingView for VControllerContract {}

    impl VControllingInternal for VControllerContract {}

    impl VControllerContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(
            measurer_address: AccountId,
            vault_address: AccountId,
            owner: AccountId,
        ) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut VControllerContract| {
                instance.control.measurer_address = measurer_address;
                instance.control.vault_address = vault_address;
                instance._init_with_owner(owner);
            })
        }
    }
}
