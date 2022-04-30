pub use crate::traits::managing::*;
use brush::declare_storage_trait;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};

pub use stable_coin_project_derive::ManagingStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct ManagingData {}

declare_storage_trait!(ManagingStorage, ManagingData);
