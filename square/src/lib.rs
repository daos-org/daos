// Copyright 2022 LISTEN TEAM.
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

// forked from https://github.com/paritytech/substrate/tree/master/frame/democracy
// Only a small portion of the democracy module's code is used here, and the functionality varies considerably.
// For better compatibility, it should be simple and easy to understand.
// You can set a minimum vote value for each call.
// If the "yes" vote is greater than the "no" vote and the minimum number of votes is met(That is, the probability of voting meets the requirement),
// the call can dispatch.

#![cfg_attr(not(feature = "std"), no_std)]

pub use codec::{Decode, Encode};
use dao::{self, AccountIdConversion, Hash, Vec};
use frame_support::dispatch::{DispatchResult as DResult, UnfilteredDispatchable};
pub use frame_support::{
	traits::{Defensive, Get},
	BoundedVec, RuntimeDebug,
};
use sp_std::boxed::Box;
use orml_traits::{arithmetic::CheckedAdd, MultiCurrency, MultiReservableCurrency, MultiCurrencyExtended};
pub use pallet::*;
use scale_info::TypeInfo;
pub use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::{
	traits::{BlockNumberProvider, CheckedMul},
	DispatchError,
};
pub use sp_std::{fmt::Debug, result};
pub use traits::*;
use primitives::constant::weight::DAOS_BASE_WEIGHT;

pub type AssetId = u32;
pub type PropIndex = u32;
pub type ReferendumIndex = u32;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod traits;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Tally<Balance> {
	/// The number of aye votes, expressed in terms of post-conviction lock-vote.
	pub ayes: Balance,
	/// The number of nay votes, expressed in terms of post-conviction lock-vote.
	pub nays: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Attitude {
	AYES,
	NAYS,
}

impl Default for Attitude {
	fn default() -> Self {
		Self::AYES
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct VoteInfo<SecondId, Vote, BlockNumber, VoteWeight, Attitude, ReferendumIndex> {
	second_id: SecondId,
	vote: Vote,
	attitude: Attitude,
	vote_weight: VoteWeight,
	unlock_block: BlockNumber,
	referendum_index: ReferendumIndex,
}

/// Info regarding an ongoing referendum.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ReferendumStatus<BlockNumber, Call, Balance> {
	/// When voting on this referendum will end.
	pub end: BlockNumber,
	/// The hash of the proposal being voted on.
	pub proposal: Call,
	/// The delay (in blocks) to wait after a successful referendum before deploying.
	pub delay: BlockNumber,
	/// The current tally of votes in this referendum.
	pub tally: Tally<Balance>,
}

/// Info regarding a referendum, present or past.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ReferendumInfo<BlockNumber, Call, Balance> {
	/// Referendum is happening, the arg is the block number at which it will end.
	Ongoing(ReferendumStatus<BlockNumber, Call, Balance>),
	/// Referendum finished at `end`, and has been `approved` or rejected.
	Finished { approved: bool, end: BlockNumber },
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use dao::BaseDaoCallFilter;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::{pallet_prelude::*};

	pub(crate) type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Vote: Clone
			+ Default
			+ Copy
			+ Parameter
			+ Member
			+ Vote<
				BalanceOf<Self>,
				Self::AccountId,
				Self::SecondId,
				Self::Conviction,
				Self::BlockNumber,
				DispatchError,
			> + CheckedVote<Self::SecondId, DispatchError>;

		type Conviction: Clone
			+ Default
			+ Copy
			+ Parameter
			+ ConvertInto<Self::BlockNumber>
			+ ConvertInto<BalanceOf<Self>>;

		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId>
			+ MultiReservableCurrency<Self::AccountId>
			 + MultiCurrencyExtended<Self::AccountId>;

		#[pallet::constant]
		type GetNativeCurrencyId: Get<AssetId>;

		#[pallet::constant]
		type MaxPublicProps: Get<u32>;

		#[pallet::constant]
		type LaunchPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type MinimumDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type VotingPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type ReservePeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type EnactmentPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn public_prop_count)]
	pub type PublicPropCount<T: Config> = StorageMap<_, Identity, T::DaoId, PropIndex, ValueQuery>;

	/// The public proposals. Unsorted. The second item is the proposal's hash.
	#[pallet::storage]
	#[pallet::getter(fn public_props)]
	pub type PublicProps<T: Config> = StorageMap<
		_,
		Identity,
		T::DaoId,
		Vec<(PropIndex, T::Hash, <T as dao::Config>::Call, T::AccountId)>,
		ValueQuery,
	>;

	/// Those who have locked a deposit.
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
	#[pallet::storage]
	#[pallet::getter(fn deposit_of)]
	pub type DepositOf<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::DaoId,
		Identity,
		PropIndex,
		(Vec<T::AccountId>, BalanceOf<T>),
	>;

	#[pallet::storage]
	#[pallet::getter(fn reserve_of)]
	pub type ReserveOf<T: Config> =
		StorageMap<_, Identity, T::AccountId, Vec<(BalanceOf<T>, T::BlockNumber)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_info)]
	pub type ReferendumInfoOf<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::DaoId,
		Identity,
		ReferendumIndex,
		ReferendumInfo<T::BlockNumber, <T as dao::Config>::Call, BalanceOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_count)]
	pub type ReferendumCount<T: Config> =
		StorageMap<_, Identity, T::DaoId, ReferendumIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn votes_of)]
	pub type VotesOf<T: Config> = StorageMap<
		_,
		Identity,
		T::AccountId,
		Vec<
			VoteInfo<T::SecondId, T::Vote, T::BlockNumber, BalanceOf<T>, Attitude, ReferendumIndex>,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn min_vote_weight_of)]
	pub type MinVoteWeightOf<T: Config> = StorageDoubleMap<_, Identity, T::DaoId, Identity, T::CallId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn launch_tag)]
	pub type LaunchTag<T: Config> = StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Proposed(T::DaoId, T::Hash),
		Second(T::DaoId, BalanceOf<T>),
		StartTable(T::DaoId, ReferendumIndex),
		Vote(T::DaoId, ReferendumIndex, T::Vote),
		CancelVote(T::DaoId, ReferendumIndex),
		EnactProposal { dao_id: T::DaoId, index: ReferendumIndex, result: DResult },
		Unlock(T::AccountId, T::SecondId, T::Vote),
		Unreserved(T::AccountId, BalanceOf<T>),
		SetMinVoteWeight(T::DaoId, T::CallId, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		Overflow,
		ValueLow,
		TooManyProposals,
		ProposalMissing,
		NoneWaiting,
		ReferendumNotExists,
		ReferendumFinished,
		VoteNotEnd,
		InDelayTime,
		VoteEnd,
		VoteEndButNotPass,
		VoteNotExists,
		NotTableTime,
		VoteError,
		VoteNotEnough,
		VoteWeightTooLow,

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn propose(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: Box<<T as dao::Config>::Call>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				dao::Pallet::<T>::get_second_id(dao_id)?.contains(*proposal.clone()),
				dao::Error::<T>::NotDaoSupportCall
			);
			ensure!(value >= T::MinimumDeposit::get(), Error::<T>::ValueLow);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let index = Self::public_prop_count(dao_id);
			let real_prop_count = PublicProps::<T>::decode_len(dao_id).unwrap_or(0) as u32;
			let max_proposals = T::MaxPublicProps::get();
			ensure!(real_prop_count < max_proposals, Error::<T>::TooManyProposals);

			T::MultiCurrency::reserve(T::GetNativeCurrencyId::get(), &who, value)?;

			PublicPropCount::<T>::insert(dao_id, index + 1);
			<DepositOf<T>>::insert(dao_id, index, (&[&who][..], value));

			<PublicProps<T>>::append(dao_id, (index, proposal_hash, *proposal, who));

			Self::deposit_event(Event::<T>::Proposed(dao_id, proposal_hash));
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn second(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			#[pallet::compact] proposal: PropIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut deposit =
				Self::deposit_of(dao_id, proposal).ok_or(Error::<T>::ProposalMissing)?;
			let deposit_amount = deposit.1.clone();
			T::MultiCurrency::reserve(T::GetNativeCurrencyId::get(), &who, deposit_amount)?;
			deposit.0.push(who.clone());
			<DepositOf<T>>::insert(dao_id, proposal, deposit);
			let unreserved_block =
				Self::now().checked_add(&T::ReservePeriod::get()).ok_or(Error::<T>::Overflow)?;
			ReserveOf::<T>::append(who.clone(), (deposit_amount, unreserved_block));
			Self::deposit_event(Event::<T>::Second(dao_id, deposit_amount));

			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn start_table(origin: OriginFor<T>, dao_id: T::DaoId) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let tag = LaunchTag::<T>::get(dao_id);
			let now = Self::now();
			let dao_start_time = dao::Pallet::<T>::get_dao(dao_id)?.start_block;
			// (now - dao_start_time) / LaunchPeriod > tag
			ensure!(
				tag.checked_mul(&T::LaunchPeriod::get()).ok_or(Error::<T>::Overflow)? <
					(now - dao_start_time),
				Error::<T>::NotTableTime
			);
			let index = Self::launch_public(dao_id)?;
			Self::deposit_event(Event::<T>::StartTable(dao_id, index));

			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn vote(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			index: ReferendumIndex,
			vote: T::Vote,
			conviction: T::Conviction,
			attitude: Attitude,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = Self::now();
			let mut vote_weight = BalanceOf::<T>::from(0u32);

			ReferendumInfoOf::<T>::try_mutate_exists(
				dao_id,
				index,
				|h| -> result::Result<(), DispatchError> {
					let mut info = h.take().ok_or(Error::<T>::ReferendumNotExists)?;
					if let ReferendumInfo::Ongoing(ref mut x) = info {
						if x.end > now {
							let second_id = dao::Pallet::<T>::get_second_id(dao_id)?;
							ensure!(vote.is_can_vote(second_id.clone())?, Error::<T>::VoteError);
							let vote_result = vote.try_vote(&who, &second_id, &conviction)?;
							vote_weight = vote_result.0;
							let duration = vote_result.1;
							match attitude {
								Attitude::NAYS => {
									x.tally.nays += vote_weight;
								},
								Attitude::AYES => {
									x.tally.ayes += vote_weight;
								},
							};
							VotesOf::<T>::append(
								&who,
								VoteInfo {
									second_id,
									vote,
									attitude,
									vote_weight,
									unlock_block: now + duration,
									referendum_index: index,
								},
							);
						} else {
							return Err(Error::<T>::VoteEnd)?
						}
					} else {
						return Err(Error::<T>::ReferendumFinished)?
					}
					*h = Some(info);
					Ok(())
				},
			)?;

			Self::deposit_event(Event::<T>::Vote(dao_id, index, vote));
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn cancel_vote(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			index: ReferendumIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ReferendumInfoOf::<T>::try_mutate_exists(
				dao_id,
				index,
				|h| -> result::Result<(), DispatchError> {
					let mut info = h.take().ok_or(Error::<T>::ReferendumNotExists)?;
					let now = Self::now();
					if let ReferendumInfo::Ongoing(ref mut x) = info {
						if x.end > now {
							let mut votes = VotesOf::<T>::get(&who);
							votes.retain(|h| {
								if h.referendum_index == index {
									if h.vote.vote_end_do(&who, &h.second_id).is_err() {
										true
									} else {
										match h.attitude {
											Attitude::NAYS => {
												x.tally.nays =
													x.tally.nays.saturating_sub(h.vote_weight);
											},
											_ => {
												x.tally.ayes =
													x.tally.ayes.saturating_sub(h.vote_weight);
											},
										};
										false
									}
								} else {
									true
								}
							});
							VotesOf::<T>::insert(&who, votes);
						} else {
							return Err(Error::<T>::VoteEnd)?
						}
					} else {
						return Err(Error::<T>::ReferendumFinished)?
					}
					*h = Some(info);
					Ok(())
				},
			)?;
			Self::deposit_event(Event::<T>::CancelVote(dao_id, index));

			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn enact_proposal(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			index: ReferendumIndex,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let now = Self::now();
			let mut approved = false;
			let info =
				ReferendumInfoOf::<T>::get(dao_id, index).ok_or(Error::<T>::ReferendumNotExists)?;
			match info {
				ReferendumInfo::Ongoing(x) =>
					if x.end > now {
						return Err(Error::<T>::VoteNotEnd)?
					} else {
						if x.end.saturating_add(x.delay) < now {
							return Err(Error::<T>::InDelayTime)?
						} else {
							let call_id: T::CallId = TryFrom::<<T as dao::Config>::Call>::try_from(x.proposal.clone())
								.ok()
								.ok_or(dao::Error::<T>::HaveNoCallId)?;

							if x.tally.ayes.saturating_add(x.tally.nays) >= MinVoteWeightOf::<T>::get(dao_id, call_id) {
								if x.tally.ayes > x.tally.nays {
								approved = true;
								let res = x.proposal.dispatch_bypass_filter(
									frame_system::RawOrigin::Signed(
										dao::Pallet::<T>::get_second_id(dao_id)?.into_account(),
									)
									.into(),
								);
								Self::deposit_event(Event::EnactProposal {
									dao_id,
									index,
									result: res.map(|_| ()).map_err(|e| e.error),
								});
							} else {
								Self::deposit_event(Event::EnactProposal {
									dao_id,
									index,
									result: Err(Error::<T>::VoteEndButNotPass)?,
								});
							}
							} else {
								return Err(Error::<T>::VoteWeightTooLow)?;
							}

						}
					},
				_ => return Err(Error::<T>::ReferendumFinished)?,
			}
			ReferendumInfoOf::<T>::insert(
				dao_id,
				index,
				ReferendumInfo::Finished { approved, end: now },
			);

			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn unlock(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = Self::now();
			let mut votes = VotesOf::<T>::get(&who);
			ensure!(votes.len() > 0, Error::<T>::VoteNotExists);
			votes.retain(|h| {
				if h.unlock_block > now {
					true
				} else {
					if h.vote.vote_end_do(&who, &h.second_id).is_err() {
						true
					} else {
						Self::deposit_event(Event::<T>::Unlock(
							who.clone(),
							h.second_id.clone(),
							h.vote,
						));
						false
					}
				}
			});
			VotesOf::<T>::insert(&who, votes);
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn unreserve(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut total = BalanceOf::<T>::from(0u32);
			let now = Self::now();
			let mut reserve_info = ReserveOf::<T>::get(&who);
			reserve_info.retain(|h| {
				if h.1 > now {
					true
				} else {
					T::MultiCurrency::unreserve(T::GetNativeCurrencyId::get(), &who, h.0);
					total += h.0;
					false
				}
			});
			ReserveOf::<T>::insert(&who, reserve_info);
			Self::deposit_event(Event::<T>::Unreserved(who, total));

			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_min_vote_weight_for_every_call(origin: OriginFor<T>, dao_id: T::DaoId, call: Box<<T as dao::Config>::Call>, min_vote_weight: BalanceOf<T>) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			ensure!(
				dao::Pallet::<T>::get_second_id(dao_id)?.contains(*call.clone()),
				dao::Error::<T>::NotDaoSupportCall
			);
			let call_id: T::CallId = TryFrom::<<T as dao::Config>::Call>::try_from(*call.clone())
				.ok()
				.ok_or(dao::Error::<T>::HaveNoCallId)?;
			MinVoteWeightOf::<T>::insert(dao_id, call_id, min_vote_weight);
			Self::deposit_event(Event::<T>::SetMinVoteWeight(dao_id, call_id, min_vote_weight));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn backing_for(dao_id: T::DaoId, proposal: PropIndex) -> Option<BalanceOf<T>> {
		Self::deposit_of(dao_id, proposal).map(|(l, d)| d.saturating_mul((l.len() as u32).into()))
	}

	fn launch_public(dao_id: T::DaoId) -> result::Result<ReferendumIndex, DispatchError> {
		let mut public_props = Self::public_props(dao_id);
		if let Some((winner_index, _)) = public_props
			.iter()
			.enumerate()
			.max_by_key(|x| Self::backing_for(dao_id, (x.1).0).defensive_unwrap_or_else(Zero::zero))
		{
			let now = Self::now();
			let (prop_index, _, proposal, _) = public_props.swap_remove(winner_index);
			<PublicProps<T>>::insert(dao_id, public_props);

			if let Some(_) = <DepositOf<T>>::take(dao_id, prop_index) {
				Ok(Self::inject_referendum(
					dao_id,
					now.saturating_add(T::VotingPeriod::get()),
					proposal,
					T::EnactmentPeriod::get(),
				))
			} else {
				Err(Error::<T>::NoneWaiting)?
			}
		} else {
			Err(Error::<T>::NoneWaiting)?
		}
	}

	fn inject_referendum(
		dao_id: T::DaoId,
		end: T::BlockNumber,
		proposal: <T as dao::Config>::Call,
		delay: T::BlockNumber,
	) -> ReferendumIndex {
		let ref_index = Self::referendum_count(dao_id);
		ReferendumCount::<T>::insert(dao_id, ref_index + 1);
		let status = ReferendumStatus { end, proposal, delay, tally: Default::default() };

		let item = ReferendumInfo::Ongoing(status);
		<ReferendumInfoOf<T>>::insert(dao_id, ref_index, item);
		ref_index
	}

	fn now() -> T::BlockNumber {
		frame_system::Pallet::<T>::current_block_number()
	}
}
