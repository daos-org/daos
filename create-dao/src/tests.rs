#![allow(unused_imports)]
#![cfg(test)]
use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, debug, log::debug};
use primitives::ids::Nft;

pub const ALICE: u64 = 1;

pub fn create_dao() {
	Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

#[test]
pub fn create_dao_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 60]).is_err());
		assert_ok!(Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]));
		assert!(Daos::<Test>::get(0u64).is_some());
		assert!(NextDaoId::<Test>::get() == 1u64);
	});
}

#[test]
pub fn dao_remark_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::dao_remark(Origin::signed(ALICE), 0u64, vec![1; 10]).is_err());
		create_dao();
		assert_ok!(Pallet::<Test>::dao_remark(
			Origin::signed(Daos::<Test>::get(0u64).unwrap().dao_account_id),
			0u64,
			vec![1; 10]
		));
	});
}

#[test]
pub fn get_creator() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::try_get_creator(0u64).is_err());
		create_dao();
		assert_ok!(Pallet::<Test>::try_get_creator(0u64));
	});
}

#[test]
pub fn get_dao() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::try_get_dao(0u64).is_err());
		create_dao();
		assert_ok!(Pallet::<Test>::try_get_dao(0u64));
	});
}

#[test]
pub fn get_concrete_id() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::try_get_concrete_id(0u64).is_err());
		create_dao();
		assert_ok!(Pallet::<Test>::try_get_concrete_id(0u64));
	});
}

#[test]
pub fn get_dao_account_id() {
	new_test_ext().execute_with(|| {
		assert!(Pallet::<Test>::try_get_dao_account_id(0u64).is_err());
		create_dao();
		assert_ok!(Pallet::<Test>::try_get_dao_account_id(0u64));
	});
}
