#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub mod impls;
pub mod traits;
pub use stable_coin_project_derive::CollaterallingStorage;
pub use stable_coin_project_derive::EatingStorage;
pub use stable_coin_project_derive::EmittingStorage;
pub use stable_coin_project_derive::SControllingStorage;
pub use stable_coin_project_derive::SPControllingStorage;
pub use stable_coin_project_derive::SPGeneratingStorage;
pub use stable_coin_project_derive::VControllingStorage;
pub use stable_coin_project_derive::VEatingStorage;
