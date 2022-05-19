#![cfg_attr(not(feature = "std"), no_std)]

extern crate proc_macro;

use brush_derive::declare_derive_storage_trait;

declare_derive_storage_trait!(
    derive_minting_storage,
    SMintingStorage,
    SMintingStorageField
);
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
declare_derive_storage_trait!(
    derive_vault_controlling_storage,
    VControllingStorage,
    VControllingStorageField
);

declare_derive_storage_trait!(
    derive_stable_controlling_storage,
    SControllingStorage,
    SControllingStorageField
);

declare_derive_storage_trait!(
    derive_measuring_storage,
    MeasuringStorage,
    MeasuringStorageField
);

declare_derive_storage_trait!(
    derive_oracling_storage,
    OraclingStorage,
    OraclingStorageField
);

declare_derive_storage_trait!(
    derive_shares_profit_controlling_storage,
    SPControllingStorage,
    SPControllingStorageField
);

declare_derive_storage_trait!(
    derive_profit_generating_storage,
    PGeneratingStorage,
    PGeneratingStorageField
);

declare_derive_storage_trait!(
    derive_shares_profit_generating_storage,
    SPGeneratingStorage,
    SPGeneratingStorageField
);
