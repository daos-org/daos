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

// forked from https://github.com/paritytech/substrate/tree/master/frame/collective.
// Much of the code here comes from substrate's collective module.
// We can get a collective from every dao and vote to do something, and set ensure origin for every call freely.
//
// Remove EnsureMember, EnsureMembers, EnsureProportionAtLeast, EnsureProportionMoreThan, and so on.
// In their place is the EnsureOriginWithArg.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]
#![allow(clippy::tabs_in_doc_comments)]

//! # Agency Module
//!
//! ## Module Introduction
//! Agency module is the power agency in DAO, it can handle things that need to be decided quickly and efficiently.
//! Also, this module provides a method `set_ensure_origin_for_every_call` to set the Origin for each external transaction,
//! and the Agency executes the external transaction according to the Origin.
//!
//! Note that this module can only indirectly call external transactions through the doas module,
//! so whether it is the `execute` or `propose` function in the module, the value of the proposal parameter should be the `do_as_agency method`.
//! The `do_as_agency` method is a method in the doas module.
//!
//! Below is a test code case that agency calls the `set_max_members` method.
//! ***
//! let set_max_members =
//! 			Call::Agency(crate::Call::set_max_members { dao_id: 0u64, max: 100u32 });
//!
//! let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency {
//! 			dao_id: 0u64,
//! 			call: Box::new(set_max_members),
//! 		});
//!
//! assert!(crate::Pallet::<Test>::execute(
//! 			Origin::signed(ALICE),
//! 			0u64,
//! 			Box::new(do_as_agency)
//! 		)
//! 		.is_ok());
//! ***

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, PostDispatchInfo, GetDispatchInfo},
	ensure,
	traits::{Get, StorageVersion},
	weights::{Weight},
};
pub use pallet::*;
use primitives::{
	traits::{EnsureOriginWithArg, SetCollectiveMembers},
	types::{DoAsEnsureOrigin, MemberCount, Proportion, ProposalIndex},
};

use frame_support::sp_runtime::traits::Hash;
use frame_support::pallet_prelude::DispatchError;
pub use scale_info::{prelude::boxed::Box, TypeInfo};
use sp_runtime::{RuntimeDebug, traits::Dispatchable};
use sp_std::{marker::PhantomData, prelude::*, result};
use weights::WeightInfo;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod traits;
pub mod weights;

/// Default voting strategy when a member is inactive.
pub trait DefaultVote {
	/// Get the default voting strategy, given:
	///
	/// - Whether the prime member voted Aye.
	/// - Raw number of yes votes.
	/// - Raw number of no votes.
	/// - Total number of member count.
	fn default_vote(
		prime_vote: Option<bool>,
		yes_votes: MemberCount,
		no_votes: MemberCount,
		len: MemberCount,
	) -> bool;
}

/// Set the prime member's vote as the default vote.
pub struct PrimeDefaultVote;

impl DefaultVote for PrimeDefaultVote {
	fn default_vote(
		prime_vote: Option<bool>,
		_yes_votes: MemberCount,
		_no_votes: MemberCount,
		_len: MemberCount,
	) -> bool {
		prime_vote.unwrap_or(false)
	}
}

/// First see if yes vote are over majority of the whole collective. If so, set the default vote
/// as yes. Otherwise, use the prime member's vote as the default vote.
pub struct MoreThanMajorityThenPrimeDefaultVote;

impl DefaultVote for MoreThanMajorityThenPrimeDefaultVote {
	fn default_vote(
		prime_vote: Option<bool>,
		yes_votes: MemberCount,
		_no_votes: MemberCount,
		len: MemberCount,
	) -> bool {
		let more_than_majority = yes_votes * 2 > len;
		more_than_majority || prime_vote.unwrap_or(false)
	}
}

/// Origin for the collective module.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(I))]
pub enum RawOrigin<DaoId, I> {
	/// It has been condoned by a given number of members of the collective from a given total.
	Members(DaoId, MemberCount, MemberCount),
	/// It has been condoned by a single member of the collective.
	Member(DaoId),
	/// Collective does not have execute permission.
	Root(DaoId),
	/// Dummy to manage the fact we have instancing.
	_Phantom(PhantomData<I>),
}

/// Info for keeping track of a motion being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Votes<AccountId, BlockNumber> {
	/// The proposal's unique index.
	index: ProposalIndex,
	/// The number of approval votes that are needed to pass the motion.
	threshold: MemberCount,
	/// The current set of voters that approved it.
	ayes: Vec<AccountId>,
	/// The current set of voters that rejected it.
	nays: Vec<AccountId>,
	/// The hard end time of this vote.
	end: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::Contains};
	use frame_system::pallet_prelude::*;
	// use primitives::traits::BaseCallFilter;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + dao::Config {
		/// The outer origin type.
		type Origin: From<RawOrigin<<Self as dao::Config>::DaoId, I>>
			+ Into<
				Result<
					RawOrigin<<Self as dao::Config>::DaoId, I>,
					<Self as pallet::Config<I>>::Origin,
				>,
			>;

		/// The outer event type.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The outer call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<RuntimeOrigin = <Self as Config<I>>::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ From<Call<Self, I>>
			+ From<dao::Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ GetDispatchInfo;

		/// External transactions that collectives can execute directly.
		type CollectiveBaseCallFilter: Contains<Self::Proposal>;

		/// Default vote strategy of this collective.
		type DefaultVote: DefaultVote;

		/// Collective in DAO Maximum number of people.
		#[pallet::constant]
		type MaxMembersForSystem: Get<MemberCount>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Origin for the collective pallet.
	#[pallet::origin]
	pub type Origin<T, I = ()> = RawOrigin<<T as dao::Config>::DaoId, I>;

	/// The hashes of the active proposals.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::DaoId, Vec<T::Hash>, ValueQuery>;

	/// The origin of each call.
	#[pallet::storage]
	#[pallet::getter(fn ensures)]
	pub type EnsureOrigins<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128,
		<T as dao::Config>::DaoId,
		Blake2_128Concat,
		<T as dao::Config>::CallId,
		DoAsEnsureOrigin<Proportion<MemberCount>, MemberCount>,
		ValueQuery,
	>;

	/// All members of the collective.
	#[pallet::storage]
	#[pallet::getter(fn collective_members)]
	pub type CollectiveMembers<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, <T as dao::Config>::DaoId, Vec<T::AccountId>, ValueQuery>;

	/// The prime of the collective.
	#[pallet::storage]
	#[pallet::getter(fn prime)]
	pub type Prime<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, <T as dao::Config>::DaoId, T::AccountId>;

	#[pallet::type_value]
	pub fn MotionDurationOnEmpty<T: Config<I>, I: 'static>() -> u32 {
		u32::from(500u32)
	}

	/// The time-out for council motions.
	#[pallet::storage]
	#[pallet::getter(fn motion_duration)]
	pub type MotionDuration<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::DaoId, u32, ValueQuery, MotionDurationOnEmpty<T, I>>;

	#[pallet::type_value]
	pub fn MaxProposalsOnEmpty<T: Config<I>, I: 'static>() -> ProposalIndex {
		20 as ProposalIndex
	}

	/// Maximum number of proposals allowed to be active in parallel.
	#[pallet::storage]
	#[pallet::getter(fn max_proposals)]
	pub type MaxProposals<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::DaoId, ProposalIndex, ValueQuery, MaxProposalsOnEmpty<T, I>>;

	#[pallet::type_value]
	pub fn MaxMembersOnEmpty<T: Config<I>, I: 'static>() -> MemberCount {
		10 as MemberCount
	}

	/// The maximum number of members supported by the pallet.
	#[pallet::storage]
	#[pallet::getter(fn max_members)]
	pub type MaxMembers<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::DaoId, MemberCount, ValueQuery, MaxMembersOnEmpty<T, I>>;

	/// Actual proposal for a given hash, if it's current.
	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::DaoId,
		Identity,
		T::Hash,
		<T as Config<I>>::Proposal,
		OptionQuery,
	>;

	/// Votes on a given proposal, if it is ongoing.
	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type Voting<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::DaoId,
		Identity,
		T::Hash,
		Votes<T::AccountId, u32>,
		OptionQuery,
	>;

	/// Proposals so far.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub type ProposalCount<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::DaoId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed {
			account: T::AccountId,
			proposal_index: ProposalIndex,
			proposal_hash: T::Hash,
			threshold: MemberCount,
		},
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted {
			account: T::AccountId,
			proposal_hash: T::Hash,
			voted: bool,
			yes: MemberCount,
			no: MemberCount,
		},
		/// A motion was approved by the required threshold.
		Approved { proposal_hash: T::Hash },
		/// A motion was not approved by the required threshold.
		Disapproved { proposal_hash: T::Hash },
		/// A motion was executed; result will be `Ok` if it returned without error.
		Executed { proposal_hash: T::Hash, result: DispatchResult },
		/// A single member did some action; result will be `Ok` if it returned without error.
		MemberExecuted { proposal_hash: T::Hash, result: DispatchResult },
		/// A proposal was closed because its threshold was reached or after its duration was up.
		Closed { proposal_hash: T::Hash, yes: MemberCount, no: MemberCount },
		/// Set the voting duration for a proposal in each DAO.
		SetMotionDuration { dao_id: T::DaoId, duration: u32 },
		/// Set a cap on the number of proposals in each DAO.
		SetMaxProposals { dao_id: T::DaoId, max: ProposalIndex },
		/// Set the upper limit of the number of council members in each DAO.
		SetMaxMembers { dao_id: T::DaoId, max: MemberCount },
		/// Set Origin for a method in DAO.
		SetOrigin(T::DaoId, T::CallId, DoAsEnsureOrigin<Proportion<MemberCount>, MemberCount>),
	}

	/// Old name generated by `decl_event`.
	#[deprecated(note = "use `Event` instead")]
	pub type RawEvent<T, I = ()> = Event<T, I>;

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Account is not a member
		NotMember,
		/// Duplicate proposals not allowed
		DuplicateProposal,
		/// Proposal must exist
		ProposalMissing,
		/// Mismatched index
		WrongIndex,
		/// Duplicate vote ignored
		DuplicateVote,
		/// Members are already initialized!
		AlreadyInitialized,
		/// The close call was made too early, before the end of the voting.
		TooEarly,
		/// There can only be a maximum of `MaxProposals` active proposals.
		TooManyProposals,
		/// The given weight bound for the proposal was too low.
		WrongProposalWeight,
		/// The given length bound for the proposal was too low.
		WrongProposalLength,
		/// The number of people exceeds the maximum limit
		MembersTooLarge,
		/// The proportion is more than 100%
		ProportionErr,
		/// Threshold exceeds the number of people
		ThresholdWrong,
		ThresholdTooLow,
	}

	// Note that councillor operations are assigned to the operational class.
	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Dispatch a proposal from a member using the `Member` origin.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::execute())]
		pub fn execute(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: Box<<T as Config<I>>::Proposal>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			if !cfg!(any(feature = "std", feature = "runtime-benchmarks")) {
				ensure!(
					T::CollectiveBaseCallFilter::contains(&proposal),
					dao::Error::<T>::InVailCall
				);
			}
			ensure!(Self::is_member(dao_id, &who)?, Error::<T, I>::NotMember);
			let proposal_hash = T::Hashing::hash_of(&proposal);
			let result = proposal.dispatch(RawOrigin::Member(dao_id).into());
			Self::deposit_event(Event::MemberExecuted {
				proposal_hash,
				result: result.map(|_| ()).map_err(|e| e.error),
			});

			Ok(().into())
		}

		/// Add a new proposal to either be voted on or executed directly.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::propose())]
		pub fn propose(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			#[pallet::compact] threshold: MemberCount,
			proposal: Box<<T as Config<I>>::Proposal>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			if !cfg!(any(feature = "std", feature = "runtime-benchmarks")) {
				ensure!(
					T::CollectiveBaseCallFilter::contains(&proposal),
					dao::Error::<T>::InVailCall
				);
			}
			ensure!(threshold > 1 as MemberCount, Error::<T, I>::ThresholdTooLow);
			ensure!(Self::is_member(dao_id, &who)?, Error::<T, I>::NotMember);
			let proposal_hash = T::Hashing::hash_of(&proposal);
			ensure!(
				!<ProposalOf<T, I>>::contains_key(&dao_id, proposal_hash),
				Error::<T, I>::DuplicateProposal
			);

			ensure!(
				Self::collective_members(dao_id).len() as u32 >= threshold,
				Error::<T, I>::ThresholdWrong
			);
			<Proposals<T, I>>::try_mutate(dao_id, |proposals| -> DispatchResult {
				proposals.push(proposal_hash);
				ensure!(
					proposals.len() as u32 <= MaxProposals::<T, I>::get(dao_id),
					Error::<T, I>::WrongProposalLength
				);
				Ok(())
			})?;

			let index = Self::proposal_count(dao_id);
			<ProposalCount<T, I>>::mutate(dao_id, |i| *i += 1);
			<ProposalOf<T, I>>::insert(dao_id, proposal_hash, *proposal);
			let votes = {
				let end =
					frame_system::Pallet::<T>::block_number() + MotionDuration::<T, I>::get(dao_id).into();
				// fixme
				Votes { index, threshold, ayes: vec![who.clone()], nays: vec![], end: 100 }
			};
			<Voting<T, I>>::insert(dao_id, proposal_hash, votes);

			Self::deposit_event(Event::Proposed {
				account: who,
				proposal_index: index,
				proposal_hash,
				threshold,
			});

			Ok(().into())
		}

		/// Add an aye or nay vote for the sender to the given proposal.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::vote())]
		pub fn vote(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal: T::Hash,
			#[pallet::compact] index: ProposalIndex,
			approve: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_member(dao_id, &who)?, Error::<T, I>::NotMember);

			let mut voting =
				Self::voting(dao_id, &proposal).ok_or(Error::<T, I>::ProposalMissing)?;
			ensure!(voting.index == index, Error::<T, I>::WrongIndex);

			let position_yes = voting.ayes.iter().position(|a| a == &who);
			let position_no = voting.nays.iter().position(|a| a == &who);

			if approve {
				if position_yes.is_none() {
					voting.ayes.push(who.clone());
				} else {
					return Err(Error::<T, I>::DuplicateVote.into())
				}
				if let Some(pos) = position_no {
					voting.nays.swap_remove(pos);
				}
			} else {
				if position_no.is_none() {
					voting.nays.push(who.clone());
				} else {
					return Err(Error::<T, I>::DuplicateVote.into())
				}
				if let Some(pos) = position_yes {
					voting.ayes.swap_remove(pos);
				}
			}

			let yes_votes = voting.ayes.len() as MemberCount;
			let no_votes = voting.nays.len() as MemberCount;
			Self::deposit_event(Event::Voted {
				account: who,
				proposal_hash: proposal,
				voted: approve,
				yes: yes_votes,
				no: no_votes,
			});

			Voting::<T, I>::insert(dao_id, &proposal, voting);

			Ok(().into())
		}

		/// Close a vote that is either approved, disapproved or whose voting period has ended.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::close())]
		pub fn close(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
			#[pallet::compact] index: ProposalIndex,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;

			let voting =
				Self::voting(dao_id, &proposal_hash).ok_or(Error::<T, I>::ProposalMissing)?;
			ensure!(voting.index == index, Error::<T, I>::WrongIndex);

			let mut no_votes = voting.nays.len() as MemberCount;
			let mut yes_votes = voting.ayes.len() as MemberCount;
			let seats = Self::collective_members(dao_id).len() as MemberCount;
			let approved = yes_votes >= voting.threshold;
			let disapproved = seats.saturating_sub(no_votes) < voting.threshold;
			// Allow (dis-)approving the proposal as soon as there are enough votes.
			if approved {
				let proposal = Self::validate_and_get_proposal(&proposal_hash, dao_id)?;
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let _ =
					Self::do_approve_proposal(seats, yes_votes, proposal_hash, proposal, dao_id);
				return Ok(().into())
			} else if disapproved {
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let _proposal_count = Self::do_disapprove_proposal(proposal_hash, dao_id);
				return Ok(().into())
			}

			// Only allow actual closing of the proposal after the voting period has ended.
			// fixme
			// ensure!(
			// 	frame_system::Pallet::<T>::block_number() >= voting.end,
			// 	Error::<T, I>::TooEarly
			// );

			let prime_vote = Self::prime(dao_id).map(|who| voting.ayes.iter().any(|a| a == &who));

			// default voting strategy.
			let default = T::DefaultVote::default_vote(prime_vote, yes_votes, no_votes, seats);

			let abstentions = seats - (yes_votes + no_votes);
			match default {
				true => yes_votes += abstentions,
				false => no_votes += abstentions,
			}
			let approved = yes_votes >= voting.threshold;

			if approved {
				let proposal = Self::validate_and_get_proposal(&proposal_hash, dao_id)?;
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let _ =
					Self::do_approve_proposal(seats, yes_votes, proposal_hash, proposal, dao_id);
				Ok(().into())
			} else {
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let _proposal_count = Self::do_disapprove_proposal(proposal_hash, dao_id);
				Ok(().into())
			}
		}

		/// call id:201
		///
		/// Disapprove a proposal, close, and remove it from the system, regardless of its current state.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::disapprove_proposal())]
		pub fn disapprove_proposal(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			let _proposal_count = Self::do_disapprove_proposal(proposal_hash, dao_id);
			Ok(().into())
		}

		/// call id:202
		///
		/// Set the length of time for voting on proposal.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_motion_duration())]
		pub fn set_motion_duration(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			duration: u32,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MotionDuration::<T, I>::insert(dao_id, duration);
			Self::deposit_event(Event::SetMotionDuration { dao_id, duration });
			Ok(().into())
		}

		/// call id:203
		///
		/// Set a cap on the number of agency proposals
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_max_proposals())]
		pub fn set_max_proposals(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			max: ProposalIndex,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MaxProposals::<T, I>::insert(dao_id, max);
			Self::deposit_event(Event::SetMaxProposals { dao_id, max });
			Ok(().into())
		}

		/// call id:204
		///
		/// Set the maximum number of members in the agency.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_max_members())]
		pub fn set_max_members(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			max: MemberCount,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			MaxMembers::<T, I>::insert(dao_id, max);
			Self::deposit_event(Event::SetMaxMembers { dao_id, max });
			Ok(().into())
		}

		/// call id:205
		///
		/// Set origin for a specific call.
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_ensure_origin_for_every_call())]
		pub fn set_ensure_origin_for_every_call(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			call_id: T::CallId,
			ensure: DoAsEnsureOrigin<Proportion<MemberCount>, MemberCount>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;

			if let DoAsEnsureOrigin::Proportion(x) = ensure.clone() {
				match x {
					Proportion::MoreThan(n, m) => {
						ensure!(n <= m, Error::<T, I>::ProportionErr);
					},
					Proportion::AtLeast(n, m) => {
						ensure!(n <= m, Error::<T, I>::ProportionErr);
					},
				}
			}

			EnsureOrigins::<T, I>::insert(dao_id, call_id, ensure.clone());
			Self::deposit_event(Event::SetOrigin(dao_id, call_id, ensure));
			Ok(().into())
		}
	}
}

/// Return the weight of a dispatch call result as an `Option`.
///
/// Will return the weight regardless of what the state of the result is.
fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
	match result {
		Ok(post_info) => post_info.actual_weight,
		Err(err) => err.post_info.actual_weight,
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Check whether `who` is a member of the collective.
	pub fn is_member(dao_id: T::DaoId, who: &T::AccountId) -> result::Result<bool, DispatchError> {
		// Note: The dispatchables *do not* use this to check membership so make sure
		// to update those if this is changed.
		let members = Self::collective_members(dao_id);
		Ok(members.contains(who))
	}

	/// Ensure that the right proposal bounds were passed and get the proposal from storage.
	///
	/// Checks the length in storage via `storage::read` which adds an extra `size_of::<u32>() == 4`
	/// to the length.
	fn validate_and_get_proposal(
		hash: &T::Hash,
		dao_id: T::DaoId,
	) -> Result<<T as Config<I>>::Proposal, DispatchError> {
		let proposal =
			ProposalOf::<T, I>::get(dao_id, hash).ok_or(Error::<T, I>::ProposalMissing)?;
		Ok(proposal)
	}

	fn do_approve_proposal(
		seats: MemberCount,
		yes_votes: MemberCount,
		proposal_hash: T::Hash,
		proposal: <T as Config<I>>::Proposal,
		dao_id: T::DaoId,
	) -> (Weight, u32) {
		Self::deposit_event(Event::Approved { proposal_hash });

		let dispatch_weight = proposal.get_dispatch_info().weight;
		let origin =
			RawOrigin::Members(<T as dao::Config>::DaoId::default(), yes_votes, seats).into();
		let result = proposal.dispatch(origin);
		Self::deposit_event(Event::Executed {
			proposal_hash,
			result: result.map(|_| ()).map_err(|e| e.error),
		});
		// default to the dispatch info weight for safety
		let proposal_weight = get_result_weight(result).unwrap_or(dispatch_weight); // P1

		let proposal_count = Self::remove_proposal(proposal_hash, dao_id);
		(proposal_weight, proposal_count)
	}

	fn do_disapprove_proposal(proposal_hash: T::Hash, dao_id: T::DaoId) -> u32 {
		// disapproved
		Self::deposit_event(Event::Disapproved { proposal_hash });
		Self::remove_proposal(proposal_hash, dao_id)
	}

	// Removes a proposal from the pallet, cleaning up votes and the vector of proposals.
	fn remove_proposal(proposal_hash: T::Hash, dao_id: T::DaoId) -> u32 {
		// remove proposal and vote
		ProposalOf::<T, I>::remove(dao_id, &proposal_hash);
		Voting::<T, I>::remove(dao_id, &proposal_hash);
		let num_proposals = Proposals::<T, I>::mutate(dao_id, |proposals| {
			proposals.retain(|h| h != &proposal_hash);
			proposals.len() + 1 // calculate weight based on original length
		});
		num_proposals as u32
	}
}

impl<T: Config<I>, I: 'static> SetCollectiveMembers<T::AccountId, T::DaoId, DispatchError>
	for Pallet<T, I>
{
	fn set_members_sorted(
		dao_id: T::DaoId,
		members: &[T::AccountId],
		prime: Option<T::AccountId>,
	) -> Result<(), DispatchError> {
		if members.len() >
			MaxMembers::<T, I>::get(dao_id).min(T::MaxMembersForSystem::get()) as usize
		{
			return Err(Error::<T, I>::MembersTooLarge)?
		}
		// remove accounts from all current voting in motions.
		let mut members = members.to_vec();
		members.sort();
		for h in Self::proposals(dao_id).into_iter() {
			<Voting<T, I>>::mutate(dao_id, h, |v| {
				if let Some(mut votes) = v.take() {
					votes.ayes = votes
						.ayes
						.into_iter()
						.filter(|i| members.binary_search(i).is_ok())
						.collect();
					votes.nays = votes
						.nays
						.into_iter()
						.filter(|i| members.binary_search(i).is_ok())
						.collect();
					*v = Some(votes);
				}
			});
		}
		if let Some(p) = prime {
			Prime::<T, I>::insert(dao_id, p);
		}
		CollectiveMembers::<T, I>::insert(dao_id, members);

		Ok(())
	}
}

#[allow(non_snake_case)]
impl<T: Config<I>, I: 'static>
	EnsureOriginWithArg<<T as pallet::Config<I>>::Origin, (T::DaoId, T::CallId)> for Pallet<T, I>
{
	type Success = <T as dao::Config>::DaoId;

	fn try_origin(
		o: <T as Config<I>>::Origin,
		a: &(T::DaoId, T::CallId),
	) -> Result<Self::Success, <T as Config<I>>::Origin> {
		let ensure = EnsureOrigins::<T, I>::get(a.0, a.1);
		match ensure {
			DoAsEnsureOrigin::Proportion(pro) => match pro {
				Proportion::MoreThan(N, D) => o.into().and_then(|o| match o {
					RawOrigin::Root(dao_id) if dao_id == a.0 => Ok(dao_id),
					RawOrigin::Members(dao_id, n, m) if dao_id == a.0 && n * D > N * m =>
						Ok(dao_id),
					r => Err(<T as Config<I>>::Origin::from(r)),
				}),
				Proportion::AtLeast(N, D) => o.into().and_then(|o| match o {
					RawOrigin::Root(dao_id) if dao_id == a.0 => Ok(dao_id),
					RawOrigin::Members(dao_id, n, m) if dao_id == a.0 && n * D >= N * m =>
						Ok(dao_id),
					r => Err(<T as Config<I>>::Origin::from(r)),
				}),
			},
			DoAsEnsureOrigin::Member => o.into().and_then(|o| match o {
				RawOrigin::Root(dao_id) if dao_id == a.0 => Ok(dao_id),
				RawOrigin::Member(dao_id) if dao_id == a.0 => Ok(dao_id),
				r => Err(<T as Config<I>>::Origin::from(r)),
			}),
			DoAsEnsureOrigin::Members(N) => o.into().and_then(|o| match o {
				RawOrigin::Root(dao_id) if dao_id == a.0 => Ok(dao_id),
				RawOrigin::Members(dao_id, n, _m) if dao_id == a.0 && n >= N => Ok(dao_id),
				r => Err(<T as Config<I>>::Origin::from(r)),
			}),

			_ => o.into().and_then(|o| match o {
				RawOrigin::Root(dao_id) if dao_id == a.0 => Ok(dao_id),
				r => Err(<T as Config<I>>::Origin::from(r)),
			}),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(
		a: &(<T as dao::Config>::DaoId, <T as dao::Config>::CallId),
	) -> <T as Config<I>>::Origin {
		<T as Config<I>>::Origin::from(RawOrigin::Root(a.0))
	}
}
