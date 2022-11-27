#![allow(unused_imports)]

use super::*;
use crate::{Config, Pallet as Emergency};
use dao::Call as DaoCall;
use frame_benchmarking::{
	account, benchmarks, benchmarks_instance, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::RawOrigin as SystemOrigin;
use primitives::AccountIdConversion;
use sp_runtime::SaturatedConversion;
use sp_std::vec;

const DOLLARS: u128 = 1_00_00000_00000;

fn get_alice<T: Config>() -> T::AccountId {
	let alice = account("alice", 1, 1);
	T::Currency::deposit_creating(&alice, (100000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	alice
}

fn get_bob<T: Config>() -> T::AccountId {
	let bob = account("bob", 1, 1);
	T::Currency::deposit_creating(&bob, (100000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	bob
}

fn get_dao_account<T: Config>(second_id: T::ConcreteId) -> T::AccountId {
	let who = second_id.into_account();
	T::Currency::deposit_creating(&who, (100000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	who
}

fn creat_dao<T: Config>() -> (T::DaoId, T::ConcreteId) {
	let alice = get_alice::<T>();
	let dao_id = T::DaoId::default();
	let second_id: T::ConcreteId = Default::default();
	assert!(dao::Pallet::<T>::create_dao(
		SystemOrigin::Signed(alice).into(),
		second_id,
		vec![1; 4],
	)
	.is_ok());
	(dao_id, second_id)
}

fn get_call<T: Config>(dao_id: T::DaoId) -> (<T as dao::Config>::Call, T::Hash) {
	let proposal: <T as dao::Config>::Call =
		DaoCall::<T>::dao_remark { dao_id, remark: vec![1; 20] }.into();
	(proposal.clone(), T::Hashing::hash_of(&proposal))
}

fn get_members<T: Config>() -> T::DaoId {
	let (dao_id, second_id) = creat_dao::<T>();
	let dao = get_dao_account::<T>(second_id);
	assert!(Emergency::<T>::set_members(
		SystemOrigin::Signed(dao).into(),
		dao_id,
		vec![get_alice::<T>(),]
	)
	.is_ok());
	dao_id
}

// fn external<T: Config>() -> (T::DaoId, T::Hash) {
// 	let (dao_id, _second_id) = creat_dao::<T>();
// 	let (proposal, hash) = get_call::<T>(dao_id);
// 	assert!(Emergency::<T>::external_track(
// 		SystemOrigin::Root.into(),
// 		dao_id,
// 		Box::new(proposal),
// 		vec![1, 2, 3, 4]
// 	)
// 	.is_ok());
// 	(dao_id, hash)
// }

fn internal<T: Config>() -> (T::DaoId, T::Hash) {
	let dao_id = get_members::<T>();
	let (proposal, hash) = get_call::<T>(dao_id);
	assert!(Emergency::<T>::internal_track(
		SystemOrigin::Signed(get_alice::<T>()).into(),
		dao_id,
		Box::new(proposal),
		vec![1, 2, 3, 4]
	)
	.is_ok());
	(dao_id, hash)
}

benchmarks! {
	set_members {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);

	}:_(SystemOrigin::Signed(dao), dao_id, vec![get_alice::<T>(), ])

	set_pledge {
		let (dao_id, second_id) = creat_dao::<T>();
		let dao = get_dao_account::<T>(second_id);
	}:_(SystemOrigin::Signed(dao), dao_id, (100000 * DOLLARS).saturated_into::<BalanceOf<T>>())

	external_track {
		let (dao_id, second_id) = creat_dao::<T>();
		let (proposal, hash) = get_call::<T>(dao_id);
	}:_(SystemOrigin::Root, dao_id, Box::new(proposal), vec![1, 2, 3, 4])

	internal_track {
		let dao_id = get_members::<T>();
		let (proposal, hash) = get_call::<T>(dao_id);
	}:_(SystemOrigin::Signed(get_alice::<T>()), dao_id, Box::new(proposal), vec![1, 2, 3, 4])

	reject {
		let (dao_id, hash) = internal::<T>();
	}:_(SystemOrigin::Signed(get_bob::<T>()), dao_id, hash)

	enact_proposal {
		let (dao_id, hash) = internal::<T>();
		frame_system::Pallet::<T>::set_block_number(T::BlockNumber::from(10000u32));
	}:_(SystemOrigin::Signed(get_bob::<T>()), dao_id, hash)
}
