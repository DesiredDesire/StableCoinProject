#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub mod impls;
pub mod traits;
pub use stable_coin_project_derive::CollaterallingStorage;
pub use stable_coin_project_derive::EatingStorage;
pub use stable_coin_project_derive::EmittingStorage;
pub use stable_coin_project_derive::MinterStorage;
pub use stable_coin_project_derive::VEatingStorage;
pub use stable_coin_project_derive::VFeedingStorage;
