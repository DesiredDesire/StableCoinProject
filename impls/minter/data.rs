// importing everything publicly from traits allows you to import every stuff related to lending
// by one import
pub use crate::traits::minter::*;
use brush::{declare_storage_trait, traits::AccountId};
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
// it is public because when you will import the trait you also will import the derive for the trait
pub use stable_coin_project_derive::MinterStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct MinterData {
    pub minted_token_address: AccountId,
}

declare_storage_trait!(MinterStorage, MinterData);

// /// this internal function will be used to set price of `asset_in` when we deposit `asset_out`
// /// we are using this function in our example to simulate an oracle
// pub fn set_asset_price<T: LendingStorage>(
//     instance: &mut T,
//     asset_in: AccountId,
//     asset_out: AccountId,
//     price: Balance,
// ) {
//     instance
//         .get_mut()
//         .asset_price
//         .insert((&asset_in, &asset_out), &price);
// }

// /// this internal function will be used to set price of `asset_in` when we deposit `asset_out`
// /// we are using this function in our example to simulate an oracle
// pub fn get_asset_price<T: LendingStorage>(
//     instance: &T,
//     amount_in: Balance,
//     asset_in: AccountId,
//     asset_out: AccountId,
// ) -> Balance {
//     let price = instance
//         .get()
//         .asset_price
//         .get((&asset_in, &asset_out))
//         .unwrap_or(0);
//     price * amount_in
// }

// /// Internal function which will return the address of the shares token
// /// which are minted when `asset_address` is borrowed
// pub fn get_reserve_asset<T: LendingStorage>(
//     instance: &T,
//     asset_address: &AccountId,
// ) -> Result<AccountId, LendingError> {
//     let reserve_asset = instance
//         .get()
//         .asset_shares
//         .get(&asset_address)
//         .unwrap_or(ZERO_ADDRESS.into());
//     if reserve_asset.is_zero() {
//         return Err(LendingError::AssetNotSupported);
//     }
//     Ok(reserve_asset)
// }

// /// internal function which will return the address of asset
// /// which is bound to `shares_address` shares token
// pub fn get_asset_from_shares<T: LendingStorage>(
//     instance: &T,
//     shares_address: AccountId,
// ) -> Result<AccountId, LendingError> {
//     let token = instance
//         .get()
//         .shares_asset
//         .get(&shares_address)
//         .unwrap_or(ZERO_ADDRESS.into());
//     if token.is_zero() {
//         return Err(LendingError::AssetNotSupported);
//     }
//     Ok(token)
// }
