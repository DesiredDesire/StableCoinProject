// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::profit_controlling::*;
use brush::{declare_storage_trait, traits::AccountId};
use ink_storage::{
    traits::{SpreadAllocate, SpreadLayout},
    Mapping,
};
// it is public because when you will import the trait you also will import the derive for the trait
pub use stable_coin_project_derive::PControllingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct PControllingData {
    // immutables
    pub profit_psp22: AccountId,

    // mutables_internal;
    pub total_profit: i128,

    // mutables_external
    pub is_generator: Mapping<AccountId, bool>,
    pub treassury_address: AccountId,
    pub treassury_part_e6: u128,
}

declare_storage_trait!(PControllingStorage, PControllingData);
