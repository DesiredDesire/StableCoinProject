#![cfg_attr(not(feature = "std"), no_std)]

extern crate proc_macro;

use brush_derive::declare_derive_storage_trait;

declare_derive_storage_trait!(derive_minter_storage, MinterStorage, MinterStorageField);
declare_derive_storage_trait!(
    derive_collateralling_storage,
    CollaterallingStorage,
    CollaterallingStorageField
);
declare_derive_storage_trait!(
    derive_emitting_storage,
    EmittingStorage,
    EmittingStorageField
);
declare_derive_storage_trait!(derive_eating_storage, EatingStorage, EatingStorageField);
declare_derive_storage_trait!(
    derive_vault_eating_storage,
    VEatingStorage,
    VEatingStorageField
);
