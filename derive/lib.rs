#![cfg_attr(not(feature = "std"), no_std)]

extern crate proc_macro;

use brush_derive::declare_derive_storage_trait;

declare_derive_storage_trait!(derive_minter_storage, MinterStorage, MinterStorageField);
declare_derive_storage_trait!(derive_vault_storage, VaultStorage, VaultStorageField);
