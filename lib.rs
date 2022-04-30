#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub mod impls;
pub mod traits;
pub use stable_coin_project_derive::EmitingStorage;
pub use stable_coin_project_derive::MinterStorage;
