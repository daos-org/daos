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

pub fn set_sudo() {
	create_dao();
	let proposal = Call::Sudo(sudo::Call::set_sudo_account { dao_id: 0u64, sudo_account: ALICE });
	sudo::Pallet::<Test>::sudo(
		Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id),
		0u64,
		Box::new(proposal),
	)
	.unwrap();
	assert_eq!(sudo::Account::<Test>::get(0u64), Some(1u64));
}

pub fn sudo_set_xxx() {
	set_sudo();
	let set_max_public_props =
		Call::Square(crate::Call::set_max_public_props { dao_id: 0u64, max: 100u32 });

	let set_launch_period =
		Call::Square(crate::Call::set_launch_period { dao_id: 0u64, period: 1000u64 });

	let set_minimum_deposit =
		Call::Square(crate::Call::set_minimum_deposit { dao_id: 0u64, min: 100u64 });

	let set_voting_period =
		Call::Square(crate::Call::set_voting_period { dao_id: 0u64, period: 1000u64 });

	let set_rerserve_period =
		Call::Square(crate::Call::set_rerserve_period { dao_id: 0u64, period: 1000u64 });

	let set_enactment_period =
		Call::Square(crate::Call::set_enactment_period { dao_id: 0u64, period: 1000u64 });

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_max_public_props)
	));

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_launch_period)
	));

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_minimum_deposit)
	));

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_voting_period)
	));

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_rerserve_period)
	));

	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(ALICE),
		0u64,
		Box::new(set_enactment_period)
	));
}

pub fn propose() {
	create_dao();
	frame_system::Pallet::<Test>::set_block_number(10000);
	assert!(crate::Pallet::<Test>::open_table(Origin::signed(ALICE), 0u64).is_err());
	frame_system::Pallet::<Test>::set_block_number(0);
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
	assert!(crate::Pallet::<Test>::open_table(Origin::signed(ALICE), 0u64).is_err());
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
	assert_ok!(crate::Pallet::<Test>::vote_for_referendum(
		Origin::signed(ALICE),
		0u64,
		0u32,
		Vote(100u64),
		(),
		Opinion::AYES,
	));
	assert_ok!(crate::Pallet::<Test>::vote_for_referendum(
		Origin::signed(ALICE),
		0u64,
		0u32,
		Vote(100u64),
		(),
		Opinion::NAYS,
	));
	frame_system::Pallet::<Test>::set_block_number(20000);
	assert!(crate::Pallet::<Test>::vote_for_referendum(
		Origin::signed(ALICE),
		0u64,
		0u32,
		Vote(100u64),
		(),
		Opinion::NAYS,
	)
	.is_err());
	frame_system::Pallet::<Test>::set_block_number(10000);
}

pub fn enact() {
	vote();
	assert!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32).is_err());
	frame_system::Pallet::<Test>::set_block_number(
		10000 + VotingPeriod::<Test>::get(0u64) + EnactmentPeriod::<Test>::get(0u64) - 2,
	);
	assert!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32).is_err());
	frame_system::Pallet::<Test>::set_block_number(20000);
	let ole_min_weight = MinVoteWeightOf::<Test>::get(0u64, 0u64);
	MinVoteWeightOf::<Test>::insert(0u64, 0u64, 10000000000);
	assert!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32).is_err());
	MinVoteWeightOf::<Test>::insert(0u64, 0u64, ole_min_weight);
	assert_ok!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32));
	assert!(crate::Pallet::<Test>::enact_proposal(Origin::signed(ALICE), 0u64, 0u32).is_err());
	assert!(crate::Pallet::<Test>::vote_for_referendum(
		Origin::signed(ALICE),
		0u64,
		0u32,
		Vote(100u64),
		(),
		Opinion::NAYS,
	)
	.is_err());
	assert!(crate::Pallet::<Test>::cancel_vote(Origin::signed(ALICE), 0u64, 0u32).is_err());
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
		frame_system::Pallet::<Test>::set_block_number(20000);
		assert!(crate::Pallet::<Test>::cancel_vote(Origin::signed(ALICE), 0u64, 0u32).is_err());
		frame_system::Pallet::<Test>::set_block_number(10000);
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

#[test]
pub fn sudo_set_xxx_should_work() {
	new_test_ext().execute_with(|| {
		sudo_set_xxx();
	});
}
