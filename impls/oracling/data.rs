// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::oracling::*;
use brush::declare_storage_trait;

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
// it is public because when you will import the trait you also will import the derive for the trait
pub use stable_coin_project_derive::OraclingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct OraclingData {
    // mutables_external
    pub azero_usd_price_e6: u128,
    pub azero_ausd_price_e6: u128,
}

declare_storage_trait!(OraclingStorage, OraclingData);
