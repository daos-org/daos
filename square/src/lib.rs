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
	traits::{Currency, Defensive, Get, ReservableCurrency},
	BoundedVec, RuntimeDebug,
};
pub use pallet::*;
use primitives::constant::weight::DAOS_BASE_WEIGHT;
use scale_info::TypeInfo;
pub use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::{
	traits::{BlockNumberProvider, CheckedAdd, CheckedMul},
	DispatchError,
};
use sp_std::boxed::Box;
pub use sp_std::{fmt::Debug, result};
pub use traits::*;

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

/// Voting Statistics.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Tally<Balance> {
	/// The number of aye votes, expressed in terms of post-conviction lock-vote.
	pub ayes: Balance,
	/// The number of nay votes, expressed in terms of post-conviction lock-vote.
	pub nays: Balance,
}

/// vote yes or no
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Opinion {
	/// Agree.
	AYES,
	/// Reject.
	NAYS,
}

impl Default for Opinion {
	fn default() -> Self {
		Self::AYES
	}
}

/// Information about individual votes.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct VoteInfo<DaoId, ConcreteId, Pledge, BlockNumber, VoteWeight, Opinion, ReferendumIndex> {
	/// The id of the Dao where the vote is located.
	dao_id: DaoId,
	/// The specific group id mapped by Dao.
	concrete_id: ConcreteId,
	/// The specific thing that the vote pledged.
	pledge: Pledge,
	/// Object or agree.
	opinion: Opinion,
	/// voting weight.
	vote_weight: VoteWeight,
	/// Block height that can be unlocked.
	unlock_block: BlockNumber,
	/// The referendum id corresponding to the vote.
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
	use dao::BaseCallFilter;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// What to stake when voting in a referendum.
		type Pledge: Clone
			+ Default
			+ Copy
			+ Parameter
			+ Member
			+ Pledge<
				BalanceOf<Self>,
				Self::AccountId,
				Self::DaoId,
				Self::Conviction,
				Self::BlockNumber,
				DispatchError,
			>;
		/// The number of times the vote is magnified.
		type Conviction: Clone
			+ Default
			+ Copy
			+ Parameter
			+ ConvertInto<Self::BlockNumber>
			+ ConvertInto<BalanceOf<Self>>;
		/// Operations related to native assets.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Number of public proposals so for.
	#[pallet::storage]
	#[pallet::getter(fn public_prop_count)]
	pub type PublicPropCount<T: Config> = StorageMap<_, Identity, T::DaoId, PropIndex, ValueQuery>;

	#[pallet::type_value]
	pub fn MaxPublicPropsOnEmpty() -> PropIndex {
		100u32
	}

	/// Maximum number of public proposals at one time.
	#[pallet::storage]
	#[pallet::getter(fn max_public_props)]
	pub type MaxPublicProps<T: Config> =
		StorageMap<_, Identity, T::DaoId, u32, ValueQuery, MaxPublicPropsOnEmpty>;

	#[pallet::type_value]
	pub fn LaunchPeriodOnEmpty<T: Config>() -> T::BlockNumber {
		T::BlockNumber::from(900u32)
	}

	/// How soon can a referendum be called.
	#[pallet::storage]
	#[pallet::getter(fn launch_period)]
	pub type LaunchPeriod<T: Config> =
		StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery, LaunchPeriodOnEmpty<T>>;

	/// Minimum stake per person when making public proposals.
	#[pallet::storage]
	#[pallet::getter(fn minimum_deposit)]
	pub type MinimumDeposit<T: Config> =
		StorageMap<_, Identity, T::DaoId, BalanceOf<T>, ValueQuery>;

	#[pallet::type_value]
	pub fn VotingPeriodOnEmpty<T: Config>() -> T::BlockNumber {
		T::BlockNumber::from(900u32)
	}

	/// How long each proposal can be voted on.
	#[pallet::storage]
	#[pallet::getter(fn voting_period)]
	pub type VotingPeriod<T: Config> =
		StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery, VotingPeriodOnEmpty<T>>;

	#[pallet::type_value]
	pub fn ReservePeriodOnEmpty<T: Config>() -> T::BlockNumber {
		T::BlockNumber::from(900u32)
	}

	/// How long does it take to release the mortgage.
	#[pallet::storage]
	#[pallet::getter(fn reserve_period)]
	pub type ReservePeriod<T: Config> =
		StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery, ReservePeriodOnEmpty<T>>;

	#[pallet::type_value]
	pub fn EnactmentPeriodOnEmpty<T: Config>() -> T::BlockNumber {
		T::BlockNumber::from(900u32)
	}

	/// How soon after voting closes the proposal can be implemented.
	#[pallet::storage]
	#[pallet::getter(fn enactment_period)]
	pub type EnactmentPeriod<T: Config> =
		StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery, EnactmentPeriodOnEmpty<T>>;

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

	/// Amount of proposal locked.
	#[pallet::storage]
	#[pallet::getter(fn reserve_of)]
	pub type ReserveOf<T: Config> =
		StorageMap<_, Identity, T::AccountId, Vec<(BalanceOf<T>, T::BlockNumber)>, ValueQuery>;

	/// Referendum specific information.
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

	/// Number of referendums so far.
	#[pallet::storage]
	#[pallet::getter(fn referendum_count)]
	pub type ReferendumCount<T: Config> =
		StorageMap<_, Identity, T::DaoId, ReferendumIndex, ValueQuery>;

	/// Everyone's voting information.
	#[pallet::storage]
	#[pallet::getter(fn votes_of)]
	pub type VotesOf<T: Config> = StorageMap<
		_,
		Identity,
		T::AccountId,
		Vec<
			VoteInfo<
				T::DaoId,
				T::ConcreteId,
				T::Pledge,
				T::BlockNumber,
				BalanceOf<T>,
				Opinion,
				ReferendumIndex,
			>,
		>,
		ValueQuery,
	>;

	/// Minimum voting weight required for each external transaction.
	#[pallet::storage]
	#[pallet::getter(fn min_vote_weight_of)]
	pub type MinVoteWeightOf<T: Config> =
		StorageDoubleMap<_, Identity, T::DaoId, Identity, T::CallId, BalanceOf<T>, ValueQuery>;

	/// When the referendum was last launched.
	#[pallet::storage]
	#[pallet::getter(fn launch_tag)]
	pub type LaunchTag<T: Config> = StorageMap<_, Identity, T::DaoId, T::BlockNumber, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// initiate a proposal.
		Proposed(T::DaoId, T::Hash),
		/// Others support initiating proposals.
		Second(T::DaoId, BalanceOf<T>),
		/// Open a referendum.
		StartTable(T::DaoId, ReferendumIndex),
		/// Vote for the referendum.
		Vote(T::DaoId, ReferendumIndex, T::Pledge),
		/// Cancel a vote on a referendum.
		CancelVote(T::DaoId, ReferendumIndex),
		/// Vote and execute the transaction corresponding to the proposa.
		EnactProposal { dao_id: T::DaoId, index: ReferendumIndex, result: DResult },
		/// Unlock
		Unlock(T::AccountId, T::ConcreteId, T::Pledge),
		/// Unlock
		Unreserved(T::AccountId, BalanceOf<T>),
		/// Set Origin for each Call.
		SetMinVoteWeight(T::DaoId, T::CallId, BalanceOf<T>),
		/// Set the maximum number of proposals at the same time.
		SetMaxPublicProps { dao_id: T::DaoId, max: u32 },
		/// Set the referendum interval.
		SetLaunchPeriod { dao_id: T::DaoId, period: T::BlockNumber },
		/// Set the minimum amount a proposal needs to stake.
		SetMinimumDeposit { dao_id: T::DaoId, min: BalanceOf<T> },
		/// Set the voting length of the referendum.
		SetVotingPeriod { dao_id: T::DaoId, period: T::BlockNumber },
		/// Set the length of time that can be unreserved.
		SetReservePeriod { dao_id: T::DaoId, period: T::BlockNumber },
		/// Set the time to delay the execution of the proposal.
		SetEnactmentPeriod { dao_id: T::DaoId, period: T::BlockNumber },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Integer computation overflow.
		Overflow,
		/// Insufficient amount of deposit.
		DepositTooLow,
		/// Maximum number of proposals reached.
		TooManyProposals,
		/// Proposal does not exist.
		ProposalMissing,
		/// There are no proposals in progress.
		NoneWaiting,
		/// Referendum does not exist.
		ReferendumNotExists,
		/// Referendum ends.
		ReferendumFinished,
		/// Referendum voting is underway.
		VoteNotEnd,
		/// Delayed execution time.
		InDelayTime,
		/// Referendum voting has ended.
		VoteEnd,
		/// Voting closed but proposal rejected.
		VoteEndButNotPass,
		/// It's not time to open a new referendum.
		NotTableTime,
		/// Bad origin.
		VoteWeightTooLow,
		///
		PledgeNotEnough,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// initiate a proposal.
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn propose(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: Box<<T as dao::Config>::Call>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				dao::Pallet::<T>::try_get_concrete_id(dao_id)?.contains(*proposal.clone()),
				dao::Error::<T>::InVailCall
			);
			ensure!(value >= MinimumDeposit::<T>::get(dao_id), Error::<T>::DepositTooLow);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let index = Self::public_prop_count(dao_id);
			let real_prop_count = PublicProps::<T>::decode_len(dao_id).unwrap_or(0) as u32;
			let max_proposals = MaxPublicProps::<T>::get(dao_id);
			ensure!(real_prop_count < max_proposals, Error::<T>::TooManyProposals);

			T::Currency::reserve(&who, value)?;

			PublicPropCount::<T>::insert(dao_id, index + 1);
			<DepositOf<T>>::insert(dao_id, index, (&[&who][..], value));

			<PublicProps<T>>::append(dao_id, (index, proposal_hash, *proposal, who));

			Self::deposit_event(Event::<T>::Proposed(dao_id, proposal_hash));
			Ok(().into())
		}

		/// Others support initiating proposals.
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
			T::Currency::reserve(&who, deposit_amount)?;
			deposit.0.push(who.clone());
			<DepositOf<T>>::insert(dao_id, proposal, deposit);
			let unreserved_block = Self::now()
				.checked_add(&ReservePeriod::<T>::get(dao_id))
				.ok_or(Error::<T>::Overflow)?;
			ReserveOf::<T>::append(who.clone(), (deposit_amount, unreserved_block));
			Self::deposit_event(Event::<T>::Second(dao_id, deposit_amount));

			Ok(().into())
		}

		/// Open a referendum.
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn open_table(origin: OriginFor<T>, dao_id: T::DaoId) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let tag = LaunchTag::<T>::get(dao_id);
			let now = Self::now();
			let dao_start_time = dao::Pallet::<T>::try_get_dao(dao_id)?.start_block;
			// (now - dao_start_time) / LaunchPeriod > tag
			ensure!(
				tag.checked_mul(&LaunchPeriod::<T>::get(dao_id)).ok_or(Error::<T>::Overflow)? <
					(now - dao_start_time),
				Error::<T>::NotTableTime
			);
			let index = Self::launch_public(dao_id)?;
			Self::deposit_event(Event::<T>::StartTable(dao_id, index));

			Ok(().into())
		}

		/// Vote for the referendum
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn vote_for_referendum(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			index: ReferendumIndex,
			pledge: T::Pledge,
			conviction: T::Conviction,
			opinion: Opinion,
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
							let concrete_id = dao::Pallet::<T>::try_get_concrete_id(dao_id)?;
							let vote_result = pledge.try_vote(&who, &dao_id, &conviction)?;
							vote_weight = vote_result.0;
							let duration = vote_result.1;
							match opinion {
								Opinion::NAYS => {
									x.tally.nays += vote_weight;
								},
								Opinion::AYES => {
									x.tally.ayes += vote_weight;
								},
							};
							VotesOf::<T>::append(
								&who,
								VoteInfo {
									dao_id,
									concrete_id,
									pledge,
									opinion,
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

			Self::deposit_event(Event::<T>::Vote(dao_id, index, pledge));
			Ok(().into())
		}

		/// Cancel a vote on a referendum
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
									if h.pledge.vote_end_do(&who, &dao_id).is_err() {
										true
									} else {
										match h.opinion {
											Opinion::NAYS => {
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

		/// Vote and execute the transaction corresponding to the proposa
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
							let call_id: T::CallId =
								TryFrom::<<T as dao::Config>::Call>::try_from(x.proposal.clone())
									.unwrap_or_default();

							if x.tally.ayes.saturating_add(x.tally.nays) >=
								MinVoteWeightOf::<T>::get(dao_id, call_id)
							{
								if x.tally.ayes > x.tally.nays {
									approved = true;
									let res = x.proposal.dispatch_bypass_filter(
										frame_system::RawOrigin::Signed(
											dao::Pallet::<T>::try_get_concrete_id(dao_id)?
												.into_account(),
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
								return Err(Error::<T>::VoteWeightTooLow)?
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

		/// Unlock
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn unlock(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = Self::now();
			//
			{
				let mut total = BalanceOf::<T>::from(0u32);
				let mut reserve_info = ReserveOf::<T>::get(&who);
				reserve_info.retain(|h| {
					if h.1 > now {
						true
					} else {
						T::Currency::unreserve(&who, h.0);
						total += h.0;
						false
					}
				});
				ReserveOf::<T>::insert(&who, reserve_info);
				Self::deposit_event(Event::<T>::Unreserved(who.clone(), total));
			}

			//
			{
				let mut votes = VotesOf::<T>::get(&who);
				votes.retain(|h| {
					if h.unlock_block > now {
						true
					} else {
						if h.pledge.vote_end_do(&who, &h.dao_id).is_err() {
							true
						} else {
							Self::deposit_event(Event::<T>::Unlock(
								who.clone(),
								h.concrete_id.clone(),
								h.pledge,
							));
							false
						}
					}
				});
				VotesOf::<T>::insert(&who, votes);
			}

			Ok(().into())
		}

		/// call id:301
		///
		/// Set Origin for each Call.
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_min_vote_weight_for_every_call(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			call_id: T::CallId,
			min_vote_weight: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MinVoteWeightOf::<T>::insert(dao_id, call_id, min_vote_weight);
			Self::deposit_event(Event::<T>::SetMinVoteWeight(dao_id, call_id, min_vote_weight));

			Ok(().into())
		}

		/// call id:302
		///
		/// Set the maximum number of proposals at the same time
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_max_public_props(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			max: u32,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MaxPublicProps::<T>::insert(dao_id, max);
			Self::deposit_event(Event::<T>::SetMaxPublicProps { dao_id, max });

			Ok(().into())
		}

		/// call id:303
		///
		/// Set the referendum interval
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_launch_period(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			period: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			LaunchPeriod::<T>::insert(dao_id, period);
			Self::deposit_event(Event::<T>::SetLaunchPeriod { dao_id, period });

			Ok(().into())
		}

		/// call id:304
		///
		/// Set the minimum amount a proposal needs to stake
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_minimum_deposit(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			min: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MinimumDeposit::<T>::insert(dao_id, min);
			Self::deposit_event(Event::<T>::SetMinimumDeposit { dao_id, min });

			Ok(().into())
		}

		/// call id:305
		///
		/// Set the voting length of the referendum
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_voting_period(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			period: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			VotingPeriod::<T>::insert(dao_id, period);
			Self::deposit_event(Event::<T>::SetVotingPeriod { dao_id, period });

			Ok(().into())
		}

		/// call id:306
		///
		/// Set the length of time that can be unreserved
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_rerserve_period(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			period: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			ReservePeriod::<T>::insert(dao_id, period);
			Self::deposit_event(Event::<T>::SetReservePeriod { dao_id, period });

			Ok(().into())
		}

		/// call id:307
		///
		/// Set the time to delay the execution of the proposal
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_enactment_period(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			period: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			EnactmentPeriod::<T>::insert(dao_id, period);
			Self::deposit_event(Event::<T>::SetEnactmentPeriod { dao_id, period });

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
					now.saturating_add(VotingPeriod::<T>::get(dao_id)),
					proposal,
					EnactmentPeriod::<T>::get(dao_id),
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
