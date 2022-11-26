// Copyright 2021 DICO  Developer.
// This file is part of DICO

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use dao;
use frame_support::codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct ProposalInfo<AccountId, Hash, Amount, BlockNumber> {
	who: Option<AccountId>,
	end_block: BlockNumber,
	proposal_hash: Hash,
	enact_period: BlockNumber,
	pledge: Amount,
	reason: Vec<u8>,
}


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::biguint::Double;
	use dao::Daos;
	use crate::ProposalInfo;

	pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ExternalOrigin: EnsureOrigin<Self::Origin>;

		type Currency: Currency<Self::AccountId>;

		type MinPledge: Get<BalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> = StorageMap<_, Identity, <T as dao::Config>::DaoId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pledge_of)]
	pub type PledgeOf<T: Config> = StorageMap<_, Identity, T::DaoId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn hashes_of)]
	pub type HashesOf<T: Config> = StorageMap<_, Identity, T::DaoId, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config> = StorageDoubleMap<_, Identity, T::DaoId, Identity, T::Hash, ProposalInfo<T::AccountId, T::Hash, BalanceOf<T>, T::BlockNumber>>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_memers(origin: OriginFor<T>, dao_id: T::DaoId, members: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn external_track(origin: OriginFor<T>, dao_id: T::DaoId, proposal: <T as dao::Config>::DaoId, reason: Vec<u8>) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn internal_track(origin: OriginFor<T>, dao_id: T::DaoId, proposal: <T as dao::Config>::DaoId, reason: Vec<u8>) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reject(origin: OriginFor<T>, dao_id: T::DaoId, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn enact_proposal(origin: OriginFor<T>, dao_id: T::DaoId, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

	}
}
