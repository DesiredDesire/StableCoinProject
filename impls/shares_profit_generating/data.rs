// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::shares_profit_controlling::*;
use brush::{declare_storage_trait, traits::AccountId};
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
// it is public because when you will import the trait you also will import the derive for the trait
use ink_storage::Mapping;
pub use stable_coin_project_derive::SPGeneratingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct SPGeneratingData {
    // immutables
    pub shares_token_address: AccountId,

    // mutables_internal
    pub generated_profit: i128,
    pub shares_minting_allowance: Mapping<AccountId, u128>,

    // mutables_external
    pub shares_profit_controller_address: AccountId,
    pub sharing_part_e6: u128, // 1 USDA gives shareing_part_e6 /E6 allowance
}

declare_storage_trait!(SPGeneratingStorage, SPGeneratingData);
