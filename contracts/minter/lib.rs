#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod lending {
    use brush::contracts::pausable::*;
    use ink_storage::traits::SpreadAllocate;
    use stable_coin_project::impls::minter::*;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PausableStorage, MinterStorage)]
    pub struct MinterContract {
        #[PausableStorageField]
        pause: PausableData,
        #[MinterStorageField]
        minting: MinterData,
    }

    impl Pausable for MinterContract {}

    impl Minter for MinterContract {}

    impl MinterContract {
        /// constructor with name and symbol
        #[ink(constructor)]
        pub fn new(minted_token_address: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut MinterContract| {
                instance.minting.minted_token_address = minted_token_address;
            })
        }
    }
}
