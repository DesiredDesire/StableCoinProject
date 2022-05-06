// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::vault_feeding::*;
use brush::{declare_storage_trait, traits::AccountId};
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
// it is public because when you will import the trait you also will import the derive for the trait
pub use stable_coin_project_derive::VFeedingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct VFeedingData {
    pub protocol_state_address: AccountId,
    pub minimum_collateral_coef_step_e6: u128,
    pub interest_rate_step_e12: u128,
    pub stable_coin_interest_rate_step_e12: u128,
}

declare_storage_trait!(VFeedingStorage, VFeedingData);
