#![allow(unused_imports)]

use super::*;
use crate::{Call as CollectiveCall, Config, Pallet as Collective};
use dao::Call as DaoCall;
use daos_doas::Call as DoAsCall;
use frame_benchmarking::{
	account, benchmarks, benchmarks_instance, benchmarks_instance_pallet,
	impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::{Call as SystemCall, RawOrigin as SystemOrigin};
use primitives::AccountIdConversion;

fn get_alice<T: Config<I>, I: 'static>() -> T::AccountId {
	account("alice", 1, 1)
}

fn get_dao_account<T: Config<I>, I: 'static>(second_id: T::ConcreteId) -> T::AccountId {
	second_id.into_account()
}

fn create_dao<T: Config<I>, I: 'static>() -> (T::DaoId, T::ConcreteId) {
	let second_id = T::ConcreteId::default();
	let dao_id = T::DaoId::default();
	assert!(dao::Pallet::<T>::create_dao(
		SystemOrigin::Signed(get_alice::<T, I>()).into(),
		second_id,
		vec![1; 4],
	)
	.is_ok());
	CollectiveMembers::<T, I>::insert(
		dao_id,
		vec![get_alice::<T, I>(), get_dao_account::<T, I>(second_id.clone())],
	);
	(dao_id, second_id)
}

fn get_proposal<T: Config<I>, I: 'static>(dao_id: T::DaoId) -> (T::Proposal, T::Hash) {
	let proposal: T::Proposal = DaoCall::<T>::dao_remark { dao_id, remark: vec![1; 20] }.into();
	(proposal.clone(), T::Hashing::hash_of(&proposal))
}

fn create_proposal<T: Config<I>, I: 'static>() -> (T::DaoId, T::ConcreteId, T::Hash, ProposalIndex)
{
	let (dao_id, second_id) = create_dao::<T, I>();
	let (proposal, proposal_hash) = get_proposal::<T, I>(dao_id);
	assert!(Collective::<T, I>::propose(
		SystemOrigin::Signed(get_alice::<T, I>()).into(),
		dao_id,
		2 as ProposalIndex,
		Box::new(proposal)
	)
	.is_ok());
	(dao_id, second_id, proposal_hash, 0 as ProposalIndex)
}

fn user_vote<T: Config<I>, I: 'static>() -> (T::DaoId, T::ConcreteId, T::Hash, ProposalIndex) {
	let (dao_id, second_id, proposal_hash, index) = create_proposal::<T, I>();
	let dao_account = get_dao_account::<T, I>(second_id.clone());
	assert!(Collective::<T, I>::vote(
		SystemOrigin::Signed(dao_account).into(),
		dao_id,
		proposal_hash,
		index,
		true
	)
	.is_ok());
	(dao_id, second_id, proposal_hash, index)
}

benchmarks_instance_pallet! {
	execute {
		let alice = get_alice::<T, I>();
		let (dao_id, second_id) = create_dao::<T, I>();
		let proposal = DaoCall::<T>::dao_remark {
			dao_id: dao_id,
			remark: vec![1; 20],
		}.into();

	}:_(SystemOrigin::Signed(get_dao_account::<T, I>(second_id)), dao_id, Box::new(proposal))

	propose {
		let (dao_id, second_id) = create_dao::<T, I>();
		let (proposal, proposal_hash) = get_proposal::<T, I>(dao_id);
	}:_(SystemOrigin::Signed(get_alice::<T, I>()), dao_id, 2 as ProposalIndex, Box::new(proposal))

	vote {
		let (dao_id, second_id, proposal_hash, index) = create_proposal::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, proposal_hash, index, true)

	close {
		let (dao_id, second_id, proposal_hash, index) = user_vote::<T, I>();
	}:_(SystemOrigin::Signed(get_alice::<T, I>()), dao_id, proposal_hash, index)

	disapprove_proposal {
		let (dao_id, second_id, proposal_hash, index) = create_proposal::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, proposal_hash)

	set_motion_duration {
		let (dao_id, second_id) = create_dao::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, T::BlockNumber::from(100u32))

	set_max_proposals {
		let (dao_id, second_id) = create_dao::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, 20 as ProposalIndex)

	set_max_members {
		let (dao_id, second_id) = create_dao::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, 20 as MemberCount)

	set_ensure_origin_for_every_call {
		let (dao_id, second_id) = create_dao::<T, I>();
		let dao_account = get_dao_account::<T, I>(second_id);
	}:_(SystemOrigin::Signed(dao_account), dao_id, T::CallId::default(), DoAsEnsureOrigin::Member)
}
