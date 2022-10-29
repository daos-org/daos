//! Autogenerated weights for daos_sudo
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("local"), DB CACHE: 1024

// Executed Command:
    // target/release/tico
    // benchmark
    // pallet
    // --execution=wasm
    // --chain
    // local
    // --wasm-execution=compiled
    // --pallet=daos_sudo
    // --extrinsic=*
    // --steps=50
    // --repeat=20
    // --template=./.maintain/daos-weight-template.hbs
    // --output
    // ./pallets/daos/sudo/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for daos_sudo.
pub trait WeightInfo {
    fn sudo() -> Weight;
    fn set_sudo_account() -> Weight;
    fn close_sudo() -> Weight;
}

/// Weights for daos_sudo using the Substrate node and recommended hardware.
pub struct DaosWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for DaosWeight<T> {
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:0)
        fn sudo() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:1)
        fn set_sudo_account() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:1)
        fn close_sudo() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:0)
        fn sudo() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:1)
        fn set_sudo_account() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoSudo Account (r:1 w:1)
        fn close_sudo() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
    }