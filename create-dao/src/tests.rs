#![allow(unused_imports)]
#![cfg(test)]
use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, debug};
use frame_support::log::debug;
use primitives::ids::Nft;

pub const ALICE: u64 = 1;

pub fn create_dao() {
    Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

#[test]
pub fn create_dao_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DAO::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]));
    });
}

#[test]
pub fn dao_remark_should_work() {
    new_test_ext().execute_with(|| {
        create_dao();
        assert_ok!(Pallet::<Test>::dao_remark(Origin::signed(Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, vec![1; 10]));
    });
}