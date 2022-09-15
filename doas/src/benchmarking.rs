#![allow(unused_imports)]
#![allow(dead_code)]
use super::*;
use crate::{Config, Pallet as DoAs};
use dao::{Call as DaoCall, Pallet as Dao};
use frame_benchmarking::{
	account, benchmarks, benchmarks_instance, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::RawOrigin as SystemOrigin;
use primitives::{types::ProposalIndex, AccountIdConversion};
use sp_runtime::SaturatedConversion;
use sp_std::vec;

fn get_alice<T: Config>() -> T::AccountId {
	account("alice", 1, 1)
}

fn get_dao_account<T: Config>(second_id: T::ConcreteId) -> T::AccountId {
	second_id.into_account()
}

fn creat_dao<T: Config>() -> (T::DaoId, T::ConcreteId) {
	let alice = get_alice::<T>();
	let dao_id = T::DaoId::default();
	let second_id: T::ConcreteId = Default::default();
	assert!(
		Dao::<T>::create_dao(SystemOrigin::Signed(alice).into(), second_id, vec![1;4]).is_ok()
	);
	(dao_id, second_id)
}

fn get_call<T: Config>(dao_id: T::DaoId) -> <T as dao::Config>::Call {
	let proposal: <T as dao::Config>::Call =
		DaoCall::<T>::dao_remark { dao_id, remark: vec![1; 20] }.into();
	proposal
}

benchmarks! {
	do_as_agency {
		let (dao_id, _second_id) = creat_dao::<T>();
		let call = get_call::<T>(dao_id);
		let call_id: T::CallId= TryFrom::<<T as dao::Config>::Call>::try_from(call.clone()).map_err(|_| "no call id")?;
	}:  _<T::Origin>(T::DoAsOrigin::successful_origin(&(dao_id, call_id)), dao_id, Box::new(call))
	verify {

	}
}
