pub use crate::traits::minter::*;
use brush::{contracts::psp34::*, declare_storage_trait, traits::AccountId};
use ink_primitives::{Key, KeyPtr};
use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout};
// it is public because when you will import the trait you also will import the derive for the trait
use brush::traits::Balance;
pub use stable_coin_project_derive::VaultStorage;

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::Mapping;

pub struct SingleVaultData {
    collateral: u128,
    debt: u128,
}
//based on https://github.com/paritytech/ink/blob/eb13fd3476d5b42baaf481cdc71937d8ab7fc2c4/examples/upgradeable-contracts/delegate-calls/lib.rs
const SINGLE_VAULT_DATA_STORAGE_KEY: [u8; 32] = ink_lang::blake2x256!("SingleVaultData");

impl SpreadLayout for SingleVaultData {
    const FOOTPRINT: u64 = <u128 as SpreadLayout>::FOOTPRINT + <u128 as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let mut ptr = KeyPtr::from(Key::from(SINGLE_VAULT_DATA_STORAGE_KEY));
        Self {
            collateral: SpreadLayout::pull_spread(&mut ptr),
            debt: SpreadLayout::pull_spread(&mut ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        let mut ptr = KeyPtr::from(Key::from(SINGLE_VAULT_DATA_STORAGE_KEY));
        SpreadLayout::push_spread(&self.collateral, &mut ptr);
        SpreadLayout::push_spread(&self.debt, &mut ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        let mut ptr = KeyPtr::from(Key::from(SINGLE_VAULT_DATA_STORAGE_KEY));
        SpreadLayout::clear_spread(&self.collateral, &mut ptr);
        SpreadLayout::clear_spread(&self.debt, &mut ptr);
    }
}
#[derive(Default, Debug, SpreadAllocate, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
/// define the struct with the data that our smart contract will be using
/// this will isolate the logic of our smart contract from its storage
pub struct VaultData {
    pub collateral_by_id: Mapping<Id, Balance>,
    pub debt_by_id: Mapping<Id, Balance>,
    pub collaterall_token_address: AccountId,
    pub minimumCollateralPercentage: u128,
}

declare_storage_trait!(VaultStorage, VaultData);
