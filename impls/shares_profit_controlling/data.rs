// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::shares_profit_controlling::*;
use brush::{
    declare_storage_trait,
    traits::{AccountId, Balance},
};
use ink_storage::{
    traits::{SpreadAllocate, SpreadLayout},
    Mapping,
};
// it is public because when you will import the trait you also will import the derive for the trait
pub use stable_coin_project_derive::SPControllingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct SPControllingData {
    // immutables
    pub stable_coin_address: AccountId,

    // mutables_internal;
    pub total_profit: i128,
    pub minted_amount: Balance,

    // mutables_external
    pub is_generator: Mapping<AccountId, bool>,
    pub treassury_address: AccountId,
    pub treassury_part_e6: u128,
}

declare_storage_trait!(SPControllingStorage, SPControllingData);
