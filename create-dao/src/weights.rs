//! Autogenerated weights for daos_create_dao
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
    // --pallet=daos_create-dao
    // --extrinsic=*
    // --steps=50
    // --repeat=20
    // --template=./.maintain/daos-weight-template.hbs
    // --output
    // ./pallets/daos/create-dao/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for daos_create_dao.
pub trait WeightInfo {
    fn create_dao() -> Weight;
    fn dao_remark() -> Weight;
}

/// Weights for daos_create_dao using the Substrate node and recommended hardware.
pub struct DaosWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for DaosWeight<T> {
            // Storage: CreateDao NextDaoId (r:1 w:1)
            // Storage: DaoSudo Account (r:0 w:1)
            // Storage: CreateDao Daos (r:0 w:1)
        fn create_dao() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
        fn dao_remark() -> Weight {
        (6_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: CreateDao NextDaoId (r:1 w:1)
            // Storage: DaoSudo Account (r:0 w:1)
            // Storage: CreateDao Daos (r:0 w:1)
        fn create_dao() -> Weight {
        (17_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
        fn dao_remark() -> Weight {
        (6_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
        }
    }