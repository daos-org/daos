#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(unused_must_use)]

use super::*;
use crate::{Config, Pallet as Democracy};
use dao::Call as DaoCall;
use frame_benchmarking::{
	account, benchmarks, benchmarks_instance, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::RawOrigin as SystemOrigin;
use primitives::{types::ProposalIndex, AccountIdConversion};
use sp_runtime::SaturatedConversion;
use sp_std::vec;

const DOLLARS: u128 = 1_00_00000_00000;
const LaunchTime: u32 = 1000000;

fn get_alice<T: Config>() -> T::AccountId {
	let alice = account("alice", 1, 1);
	T::Currency::deposit_creating(
		&alice,
		(100000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	alice
}

fn get_dao_account<T: Config>(second_id: T::ConcreteId) -> T::AccountId {
	let who = second_id.into_account();
	T::Currency::deposit_creating(
		&who,
		(100000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	who
}

fn creat_dao<T: Config>() -> (T::DaoId, T::ConcreteId) {
	let alice = get_alice::<T>();
	let dao_id = T::DaoId::default();
	let second_id: T::ConcreteId = Default::default();
	assert!(dao::Pallet::<T>::create_dao(
		SystemOrigin::Signed(alice).into(),
		second_id,
		vec![1;4],
	)
	.is_ok());
	(dao_id, second_id)
}

fn get_call<T: Config>(dao_id: T::DaoId) -> (<T as dao::Config>::Call, T::Hash) {
	let proposal: <T as dao::Config>::Call =
		DaoCall::<T>::dao_remark { dao_id, remark: vec![1; 20] }.into();
	(proposal.clone(), T::Hashing::hash_of(&proposal))
}

fn create_proposal<T: Config>() -> (T::DaoId, T::ConcreteId, ProposalIndex) {
	let (dao_id, second_id) = creat_dao::<T>();
	let (proposal, _) = get_call::<T>(dao_id);
	let amount = (1000 * DOLLARS).saturated_into::<BalanceOf<T>>();
	assert!(Democracy::<T>::propose(
		SystemOrigin::Signed(get_alice::<T>()).into(),
		dao_id,
		Box::new(proposal),
		amount
	)
	.is_ok());
	frame_system::Pallet::<T>::set_block_number(T::BlockNumber::from(LaunchTime));
	(dao_id, second_id, 0 as ProposalIndex)
}

fn launch<T: Config>() -> (T::DaoId, T::ConcreteId, ProposalIndex) {
	let (dao_id, second_id, index) = create_proposal::<T>();
	let dao = get_dao_account::<T>(second_id.clone());
	Democracy::<T>::open_table(SystemOrigin::Signed(dao).into(), dao_id);
	(dao_id, second_id, 0 as ProposalIndex)
}

fn vote1<T: Config>() -> (T::DaoId, T::AccountId, ProposalIndex) {
	let (dao_id, second_id, index) = launch::<T>();
	let dao = get_dao_account::<T>(second_id);
	assert!(Democracy::<T>::vote_for_referendum(
		SystemOrigin::Signed(dao.clone()).into(),
		dao_id,
		index,
		T::Pledge::default(),
		T::Conviction::default(),
		Opinion::AYES
	)
	.is_ok());
	(dao_id, dao, index)
}

fn enact<T: Config>() -> T::AccountId {
	let (dao_id, dao_account, index) = vote1::<T>();
	frame_system::Pallet::<T>::set_block_number(T::BlockNumber::from(2 as u32 * LaunchTime));
	assert!(Democracy::<T>::enact_proposal(
		SystemOrigin::Signed(dao_account.clone()).into(),
		dao_id,
		index
	)
	.is_ok());
	dao_account
}

benchmarks! {
	propose {
		let (dao_id, second_id) = creat_dao::<T>();
		let (proposal, _) = get_call::<T>(dao_id);
		let amount = (1000 * DOLLARS).saturated_into::<BalanceOf<T>>();
	}:_(SystemOrigin::Signed(get_alice::<T>()), dao_id, Box::new(proposal), amount)

	second {
		let (dao_id, second_id, index) = create_proposal::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, index)

	open_table {
		let (dao_id, second_id, index) = create_proposal::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id)

	vote_for_referendum {
		let (dao_id, second_id, index) = launch::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, index, T::Pledge::default(), T::Conviction::default(), Opinion::AYES)

	cancel_vote {
		let (dao_id, dao_account, index) = vote1::<T>();
	}:_(SystemOrigin::Signed(dao_account), dao_id, index)

	enact_proposal {
		let (dao_id, dao_account, index) = vote1::<T>();
		frame_system::Pallet::<T>::set_block_number(T::BlockNumber::from(2 as u32 * LaunchTime));
	}:_(SystemOrigin::Signed(dao_account), dao_id, index)

	unlock {
		let account = enact::<T>();
	}:_(SystemOrigin::Signed(account))

	set_min_vote_weight_for_every_call {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, T::CallId::default(), BalanceOf::<T>::from(0u32))

	set_max_public_props {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, 100u32)

	set_launch_period {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, T::BlockNumber::from(100u32))

	set_minimum_deposit {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, BalanceOf::<T>::from(0u32))

	set_voting_period {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, T::BlockNumber::from(100u32))

	set_rerserve_period {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, T::BlockNumber::from(100u32))

	set_enactment_period {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, T::BlockNumber::from(100u32))
}
