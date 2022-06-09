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
	T::MultiCurrency::deposit(
		T::GetNativeCurrencyId::get(),
		&alice,
		(100000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	alice
}

fn get_dao_account<T: Config>(second_id: T::SecondId) -> T::AccountId {
	let who = second_id.into_account();
	T::MultiCurrency::deposit(
		T::GetNativeCurrencyId::get(),
		&who,
		(100000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	who
}

fn creat_dao<T: Config>() -> (T::DaoId, T::SecondId) {
	let alice = get_alice::<T>();
	let dao_id = T::DaoId::default();
	let second_id: T::SecondId = Default::default();
	assert!(dao::Pallet::<T>::create_dao(
		SystemOrigin::Signed(alice).into(),
		dao_id,
		second_id.clone()
	)
	.is_ok());
	(dao_id, second_id)
}

fn get_call<T: Config>(dao_id: T::DaoId) -> (<T as dao::Config>::Call, T::Hash) {
	let proposal: <T as dao::Config>::Call =
		DaoCall::<T>::dao_remark { dao_id, remark: vec![1; 20] }.into();
	(proposal.clone(), T::Hashing::hash_of(&proposal))
}

fn create_proposal<T: Config>() -> (T::DaoId, T::SecondId, ProposalIndex) {
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

fn launch<T: Config>() -> (T::DaoId, T::SecondId, ProposalIndex) {
	let (dao_id, second_id, index) = create_proposal::<T>();
	let dao = get_dao_account::<T>(second_id.clone());
	Democracy::<T>::start_table(SystemOrigin::Signed(dao).into(), dao_id);
	(dao_id, second_id, 0 as ProposalIndex)
}

fn vote1<T: Config>() -> (T::DaoId, T::AccountId, ProposalIndex) {
	let (dao_id, second_id, index) = launch::<T>();
	let dao = get_dao_account::<T>(second_id);
	assert!(Democracy::<T>::vote(
		SystemOrigin::Signed(dao.clone()).into(),
		dao_id,
		index,
		T::Vote::default(),
		T::Conviction::default(),
		Attitude::AYES
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

	start_table {
		let (dao_id, second_id, index) = create_proposal::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id)

	vote {
		let (dao_id, second_id, index) = launch::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, index, T::Vote::default(), T::Conviction::default(), Attitude::AYES)

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

	unreserve {
		let account = enact::<T>();
	}:_(SystemOrigin::Signed(account))
}
