//! Autogenerated weights for daos_collective
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-12, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("local"), DB CACHE: 1024

// Executed Command:
    // ./target/release/tico
    // benchmark
    // pallet
    // --chain
    // local
    // --steps=50
    // --repeat=20
    // --pallet=daos_collective
    // --extrinsic=*
    // --template=./.maintain/pallet-weight-template.hbs
    // --output
    // ./pallets/daos/agency/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for daos_collective.
pub trait WeightInfo {
    fn execute() -> Weight;
    fn propose() -> Weight;
    fn vote() -> Weight;
    fn close() -> Weight;
    fn disapprove_proposal() -> Weight;
    fn set_motion_duration() -> Weight;
    fn set_max_proposals() -> Weight;
    fn set_max_members() -> Weight;
    fn set_ensure_origin_for_every_call() -> Weight;
}

/// Weights for daos_collective using the Substrate node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
        fn execute() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
        }
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective ProposalOf (r:1 w:1)
            // Storage: DaoCollective Proposals (r:1 w:1)
            // Storage: DaoCollective MaxProposals (r:1 w:0)
            // Storage: DaoCollective ProposalCount (r:1 w:1)
            // Storage: DaoCollective MotionDuration (r:1 w:0)
            // Storage: DaoCollective Voting (r:0 w:1)
        fn propose() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
        }
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective Voting (r:1 w:1)
        fn vote() -> Weight {
        (13_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: DaoCollective Voting (r:1 w:1)
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective ProposalOf (r:1 w:1)
            // Storage: DaoCollective Proposals (r:1 w:1)
        fn close() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective Proposals (r:1 w:1)
            // Storage: DaoCollective Voting (r:0 w:1)
            // Storage: DaoCollective ProposalOf (r:0 w:1)
        fn disapprove_proposal() -> Weight {
        (15_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MotionDuration (r:0 w:1)
        fn set_motion_duration() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MaxProposals (r:0 w:1)
        fn set_max_proposals() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MaxMembers (r:0 w:1)
        fn set_max_members() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective EnsureOrigins (r:0 w:1)
        fn set_ensure_origin_for_every_call() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
        fn execute() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
        }
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective ProposalOf (r:1 w:1)
            // Storage: DaoCollective Proposals (r:1 w:1)
            // Storage: DaoCollective MaxProposals (r:1 w:0)
            // Storage: DaoCollective ProposalCount (r:1 w:1)
            // Storage: DaoCollective MotionDuration (r:1 w:0)
            // Storage: DaoCollective Voting (r:0 w:1)
        fn propose() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
        }
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective Voting (r:1 w:1)
        fn vote() -> Weight {
        (13_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: DaoCollective Voting (r:1 w:1)
            // Storage: DaoCollective CollectiveMembers (r:1 w:0)
            // Storage: DaoCollective ProposalOf (r:1 w:1)
            // Storage: DaoCollective Proposals (r:1 w:1)
        fn close() -> Weight {
        (20_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective Proposals (r:1 w:1)
            // Storage: DaoCollective Voting (r:0 w:1)
            // Storage: DaoCollective ProposalOf (r:0 w:1)
        fn disapprove_proposal() -> Weight {
        (15_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MotionDuration (r:0 w:1)
        fn set_motion_duration() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MaxProposals (r:0 w:1)
        fn set_max_proposals() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective MaxMembers (r:0 w:1)
        fn set_max_members() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: CreateDao Daos (r:1 w:0)
            // Storage: DaoCollective EnsureOrigins (r:0 w:1)
        fn set_ensure_origin_for_every_call() -> Weight {
        (11_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
    }