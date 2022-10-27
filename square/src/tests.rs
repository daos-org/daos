#![allow(unused_imports)]
#![cfg(test)]

use super::*;
use crate::mock::{Call, Origin, *};
use frame_support::assert_ok;
use primitives::ids::Nft;
use sp_runtime::traits::BlakeTwo256;

pub const ALICE: u64 = 1;

pub fn create_dao() {
	dao::Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

pub fn propose() {
	create_dao();
	let proposal = Call::Square(crate::Call::set_min_vote_weight_for_every_call {
		dao_id: 0u64,
		call_id: 0u64,
		min_vote_weight: 100u64,
	});
	assert_ok!(crate::Pallet::<Test>::propose(
		Origin::signed(ALICE),
		0u64,
		Box::new(proposal),
		0u64
	));
}

pub fn second() {
	propose();
	assert_ok!(crate::Pallet::<Test>::second(Origin::signed(2u64), 0u64, 0u32));
}

pub fn open_table() {
	second();
	frame_system::Pallet::<Test>::set_block_number(10000);
	assert_ok!(crate::Pallet::<Test>::open_table(Origin::signed(ALICE), 0u64));
}

pub fn vote() {
	open_table();
	assert_ok!(crate::Pallet::<Test>::vote_for_referendum(
		Origin::signed(ALICE),
		0u64,
		0u32,
		Vote(100u64),
		(),
		Opinion::AYES,
	));
}

pub fn enact() {
	vote();
	frame_system::Pallet::<Test>::set_block_number(20000);
	assert_ok!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32));
}

#[test]
pub fn propose_should_work() {
	new_test_ext().execute_with(|| {
		propose();
	});
}

#[test]
pub fn second_should_work() {
	new_test_ext().execute_with(|| second());
}

#[test]
pub fn vote_should_work() {
	new_test_ext().execute_with(|| {
		vote();
	});
}

#[test]
pub fn cancel_vote_should_work() {
	new_test_ext().execute_with(|| {
		vote();
		assert_ok!(crate::Pallet::<Test>::cancel_vote(Origin::signed(ALICE), 0u64, 0u32));
	});
}

#[test]
pub fn enact_proposal_should_work() {
	new_test_ext().execute_with(|| {
		enact();
	});
}

#[test]
pub fn unlock_should_work() {
	new_test_ext().execute_with(|| {
		enact();
		assert_ok!(crate::Pallet::<Test>::unlock(Origin::signed(ALICE)));
	});
}
