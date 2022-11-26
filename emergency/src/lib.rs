// Copyright 2022 daos-org.
// This file is part of DAOS

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

use dao;
use frame_support::codec::{Decode, Encode};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct ProposalInfo<AccountId, Call, Amount, BlockNumber> {
	who: Option<AccountId>,
	end_block: BlockNumber,
	call: Call,
	enact_period: BlockNumber,
	pledge: Amount,
	reason: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
	use crate::ProposalInfo;
	use dao::Hash;
	use frame_support::{
		dispatch::{DispatchResult as DResult, DispatchResultWithPostInfo, UnfilteredDispatchable},
		pallet,
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{BlockNumberProvider, CheckedAdd};

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ExternalOrigin: EnsureOrigin<Self::Origin>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type MinPledge: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type TrackPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type EnactPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageMap<_, Identity, <T as dao::Config>::DaoId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pledge_of)]
	pub type PledgeOf<T: Config> = StorageMap<_, Identity, T::DaoId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn hashes_of)]
	pub type HashesOf<T: Config> = StorageMap<_, Identity, T::DaoId, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::DaoId,
		Identity,
		T::Hash,
		ProposalInfo<T::AccountId, <T as dao::Config>::Call, BalanceOf<T>, T::BlockNumber>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		SetMembers {
			dao_id: T::DaoId,
			members: Vec<T::AccountId>,
		},
		Track {
			dao_id: T::DaoId,
			who: Option<T::AccountId>,
			call: <T as dao::Config>::Call,
		},
		Rejected {
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		},
		EnactProposal {
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
			res: DispatchResultWithPostInfo,
		},
		SetPledge {
			dao_id: T::DaoId,
			amount: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		ProposalAlreadyExists,
		ProposalNotExists,
		TrackEnded,
		PledgeTooLow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_members(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			members: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			Members::<T>::insert(dao_id, &members);
			Self::deposit_event(Event::SetMembers { dao_id, members });
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_pledge(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			ensure!(amount >= T::MinPledge::get(), Error::<T>::PledgeTooLow);
			PledgeOf::<T>::insert(dao_id, amount);
			Self::deposit_event(Event::SetPledge { dao_id, amount });
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn external_track(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: <T as dao::Config>::Call,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			Self::try_track_do(dao_id, proposal, None, reason)
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn internal_track(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: <T as dao::Config>::Call,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = if let Ok(_) = dao::Pallet::<T>::ensrue_dao_root(origin.clone(), dao_id) {
				None
			} else {
				Some(ensure_signed(origin.clone())?)
			};

			Self::try_track_do(dao_id, proposal, who, reason)
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reject(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
			// todo
			HashesOf::<T>::try_mutate(dao_id, |hashes| -> DispatchResultWithPostInfo {
				hashes.retain(|h| h != &proposal_hash);
				let proposal = ProposalOf::<T>::take(dao_id, proposal_hash)
					.ok_or(Error::<T>::ProposalNotExists)?;
				ensure!(Self::now() < proposal.end_block, Error::<T>::TrackEnded);
				if let Some(who) = proposal.who.clone() {
					T::Currency::slash_reserved(&who, proposal.pledge);
				}
				ProposalOf::<T>::insert(dao_id, proposal_hash, proposal);
				Self::deposit_event(Event::Rejected { dao_id, proposal_hash });
				Ok(().into())
			})
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn enact_proposal(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			HashesOf::<T>::try_mutate(dao_id, |hashes| -> DispatchResultWithPostInfo {
				hashes.retain(|h| h != &proposal_hash);
				let proposal = ProposalOf::<T>::take(dao_id, proposal_hash)
					.ok_or(Error::<T>::ProposalNotExists)?;
				ensure!(
					Self::now() >=
						proposal
							.end_block
							.checked_add(&T::EnactPeriod::get())
							.ok_or(Error::<T>::StorageOverflow)?,
					Error::<T>::TrackEnded
				);
				if let Some(who) = proposal.who.clone() {
					T::Currency::unreserve(&who, proposal.pledge);
				}
				ProposalOf::<T>::insert(dao_id, proposal_hash, proposal.clone());
				let res = proposal.call.dispatch_bypass_filter(
					frame_system::RawOrigin::Signed(dao::Pallet::<T>::try_get_dao_account_id(
						dao_id,
					)?)
					.into(),
				);

				Self::deposit_event(Event::EnactProposal { dao_id, proposal_hash, res });
				Ok(().into())
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn now() -> T::BlockNumber {
			frame_system::Pallet::<T>::current_block_number()
		}

		fn try_track_do(
			dao_id: T::DaoId,
			proposal: <T as dao::Config>::Call,
			who: Option<T::AccountId>,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let proposal_hash: T::Hash = T::Hashing::hash_of(&proposal);
			HashesOf::<T>::try_mutate(dao_id, |hashes| -> DispatchResultWithPostInfo {
				if !hashes.contains(&proposal_hash) {
					hashes.push(proposal_hash);
					let end_block = Self::now()
						.checked_add(&T::TrackPeriod::get())
						.ok_or(Error::<T>::StorageOverflow)?;
					let pledge = if let None = who {
						0u32.into()
					} else {
						T::MinPledge::get().max(PledgeOf::<T>::get(dao_id))
					};
					ProposalOf::<T>::insert(
						dao_id,
						proposal_hash,
						&ProposalInfo {
							who: who.clone(),
							end_block,
							call: proposal.clone(),
							enact_period: T::EnactPeriod::get(),
							pledge,
							reason,
						},
					);
					Self::deposit_event(Event::Track { dao_id, who, call: proposal });
					return Ok(().into())
				}
				Err(Error::<T>::ProposalAlreadyExists)?
			})
		}
	}
}
