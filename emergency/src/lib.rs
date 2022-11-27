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

#[cfg(test)]
mod mock;

pub use pallet::*;
use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use dao::Hash;
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, UnfilteredDispatchable, DispatchInfo, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use dao::{Vec, Box};
use sp_runtime::traits::{BlockNumberProvider, CheckedAdd};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

/// Specific information on emergency proposal.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct ProposalInfo<AccountId, Call, Amount, BlockNumber> {
	/// who initiated the emergency proposal.
	who: Option<AccountId>,
	/// Proposal end block height.
	end_block: BlockNumber,
	/// proposal.
	call: Call,
	/// The amount that the proposal needs to pledge.
	pledge: Amount,
	/// Reason for Proposal.
	reason: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// An external origin that can submit urgent proposals to the DAO.
		type ExternalOrigin: EnsureOrigin<Self::Origin>;
		/// Operations related to funds.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The minimum pledge amount required by the system. Each DAO cannot be lower than this value.
		#[pallet::constant]
		type MinPledge: Get<BalanceOf<Self>>;
		/// How long the proposal takes.
		#[pallet::constant]
		type TrackPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Members of the DAO who can make emergency proposals.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageMap<_, Identity, <T as dao::Config>::DaoId, Vec<T::AccountId>, ValueQuery>;

	/// The amount that needs to be pledged for internal emergency proposals.
	#[pallet::storage]
	#[pallet::getter(fn pledge_of)]
	pub type PledgeOf<T: Config> = StorageMap<_, Identity, T::DaoId, BalanceOf<T>, ValueQuery>;

	/// hash of all emergency proposals in DAO.
	#[pallet::storage]
	#[pallet::getter(fn hashes_of)]
	pub type HashesOf<T: Config> = StorageMap<_, Identity, T::DaoId, Vec<T::Hash>, ValueQuery>;

	/// Specific information for each proposal.
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
		/// Set members who can make emergency proposals.
		SetMembers {
			dao_id: T::DaoId,
			members: Vec<T::AccountId>,
		},
		/// Successfully made an emergency proposal.
		Track {
			dao_id: T::DaoId,
			who: Option<T::AccountId>,
			call: <T as dao::Config>::Call,
		},
		/// Successfully rejected an emergency proposal.
		Rejected {
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		},
		/// Execute a transaction related to an emergency proposal.
		EnactProposal {
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
			res: DispatchResultWithPostInfo,
		},
		/// Set the amount that needs to be staked for an emergency proposal.
		SetPledge {
			dao_id: T::DaoId,
			amount: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		ProposalAlreadyExists,
		ProposalNotExists,
		ProposalEnded,
		PledgeTooLow,
		NotEmergencyMembers,
		/// No permission to reject proposals.
		PermissionDenied,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set members who can make emergency proposals.
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

		/// Set the amount that needs to be pledge for an emergency proposal.
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


		/// Externally initiated an emergency proposal.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn external_track(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: Box<<T as dao::Config>::Call>,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;
			Self::try_propose(dao_id, *proposal, None, reason)
		}


		/// Member initiates an urgent proposal.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn internal_track(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: Box<<T as dao::Config>::Call>,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(Members::<T>::get(dao_id).contains(&who), Error::<T>::NotEmergencyMembers);

			Self::try_propose(dao_id, *proposal, Some(who), reason)
		}


		/// Rejected an emergency proposal.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reject(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;

			ensure!(Members::<T>::get(dao_id).contains(&who), Error::<T>::NotEmergencyMembers);

			HashesOf::<T>::try_mutate(dao_id, |hashes| -> DispatchResultWithPostInfo {
				hashes.retain(|h| h != &proposal_hash);
				let proposal = ProposalOf::<T>::take(dao_id, proposal_hash)
					.ok_or(Error::<T>::ProposalNotExists)?;

				if let None = proposal.who {
					if who != dao::Pallet::<T>::try_get_dao_account_id(dao_id)? && !Members::<T>::get(dao_id).contains(&who) {
						return Err(Error::<T>::PermissionDenied)?;
					}
				}

				ensure!(Self::now() < proposal.end_block, Error::<T>::ProposalEnded);
				if let Some(who) = proposal.who.clone() {
					T::Currency::slash_reserved(&who, proposal.pledge);
				}
				ProposalOf::<T>::insert(dao_id, proposal_hash, proposal);
				Self::deposit_event(Event::Rejected { dao_id, proposal_hash });
				Ok(().into())
			})
		}


		/// Execute a transaction related to an emergency proposal.
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
				ensure!(Self::now() >= proposal.end_block, Error::<T>::ProposalEnded);
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

		fn try_propose(
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
