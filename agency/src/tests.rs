#![allow(unused_imports)]
#![cfg(test)]

use super::*;
use crate::mock::{Call, Origin, *};
use frame_support::{assert_noop, assert_ok, debug};
use primitives::{
	ids::Nft,
	types::Proportion::{AtLeast, MoreThan},
};
use sp_runtime::traits::BlakeTwo256;
use sp_std::vec;
use sudo;

pub const ALICE: u64 = 1;

pub fn create_dao() {
	dao::Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

pub fn set_members() {
	crate::CollectiveMembers::<Test>::insert(0u64, vec![ALICE, 2u64, 3u64, 4u64])
}

pub fn set_sudo() {
	create_dao();
	set_members();
	let proposal = Call::Sudo(sudo::Call::set_sudo_account { dao_id: 0u64, sudo_account: ALICE });
	sudo::Pallet::<Test>::sudo(
		Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id),
		0u64,
		Box::new(proposal),
	)
	.unwrap();
	assert_eq!(sudo::Account::<Test>::get(0u64), Some(1u64));
}

fn set_origin_for_0() {
	set_sudo();
	let proposal = Call::Agency(crate::Call::set_ensure_origin_for_every_call {
		dao_id: 0u64,
		call_id: 0u64,
		ensure: DoAsEnsureOrigin::Member,
	});
	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id),
		0u64,
		Box::new(proposal)
	));
}

fn set_origin_for_0_1() {
	set_sudo();
	let proposal = Call::Agency(crate::Call::set_ensure_origin_for_every_call {
		dao_id: 0u64,
		call_id: 0u64,
		ensure: DoAsEnsureOrigin::Members(2u32),
	});
	assert_ok!(sudo::Pallet::<Test>::sudo(
		Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id),
		0u64,
		Box::new(proposal)
	));
}

fn set_origin_for_p() {
	set_sudo();
	let proposal = Call::Agency(crate::Call::set_ensure_origin_for_every_call {
		dao_id: 0u64,
		call_id: 0u64,
		ensure: DoAsEnsureOrigin::Proportion(MoreThan(5, 4)),
	});
	let proposal_1 = Call::Agency(crate::Call::set_ensure_origin_for_every_call {
		dao_id: 0u64,
		call_id: 0u64,
		ensure: DoAsEnsureOrigin::Proportion(AtLeast(5, 4)),
	});
	assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(proposal)));
	assert_ne!(
		EnsureOrigins::<Test>::get(0u64, 0u64),
		DoAsEnsureOrigin::Proportion(MoreThan(5, 4))
	);

	assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(proposal_1)));
	assert_ne!(
		EnsureOrigins::<Test>::get(0u64, 0u64),
		DoAsEnsureOrigin::Proportion(MoreThan(5, 4))
	);
}

#[test]
fn set_origin_for_p_should_work() {
	new_test_ext().execute_with(|| {
		set_origin_for_p();
	});
}

#[test]
fn set_origin_should_work() {
	new_test_ext().execute_with(|| {
		set_origin_for_0();
	});
}

#[test]
fn sudo_set_xxx() {
	new_test_ext().execute_with(|| {
		set_sudo();
		let set_motion_duration =
			Call::Agency(crate::Call::set_motion_duration { dao_id: 0u64, duration: 100 });

		let set_max_proposals =
			Call::Agency(crate::Call::set_max_proposals { dao_id: 0u64, max: 100 });

		sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(set_motion_duration))
			.unwrap();
		sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(set_max_proposals))
			.unwrap();
	});
}

#[test]
fn set_members_sorted_should_work() {
	new_test_ext().execute_with(|| {
		set_sudo();

		let set_max_members =
			Call::Agency(crate::Call::set_max_members { dao_id: 0u64, max: 100u32 });

		let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency {
			dao_id: 0u64,
			call: Box::new(set_max_members),
		});
		let hash = BlakeTwo256::hash_of(&do_as_agency);

		assert_ok!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		));

		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, false));
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(3u64), 0, hash, 0, true));
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(4u64), 0, hash, 0, false));

		let members: Vec<u64> = vec![3, 4, 5, 6];
		let members_1: Vec<u64> = vec![1, 2, 3, 4, 8];
		assert_ok!(crate::Pallet::<Test>::set_members_sorted(0u64, &members[..], Some(1u64),));
		assert!(
			crate::Pallet::<Test>::set_members_sorted(0u64, &members_1[..], Some(1u64),).is_err()
		);
	});
}

#[test]
fn execute() {
	new_test_ext().execute_with(|| {
		set_origin_for_0();
		let set_max_members =
			Call::Agency(crate::Call::set_max_members { dao_id: 0u64, max: 100u32 });

		let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency {
			dao_id: 0u64,
			call: Box::new(set_max_members),
		});

		let do_as_agency_fail = Call::DoAs(daos_doas::Call::do_as_agency {
			dao_id: 0u64,
			call: Box::new(do_as_agency.clone()),
		});

		assert!(crate::Pallet::<Test>::execute(
			Origin::signed(ALICE),
			0u64,
			Box::new(do_as_agency_fail)
		)
		.is_ok());

		assert_ok!(crate::Pallet::<Test>::execute(
			Origin::signed(ALICE),
			0u64,
			Box::new(do_as_agency)
		));

		assert_eq!(crate::Pallet::<Test>::max_members(0u64), 100);
	});
}

#[test]
fn proposal_vote_close_should_work() {
	new_test_ext().execute_with(|| {
		set_origin_for_0_1();
		let set_max_members =
			Call::Agency(crate::Call::set_max_members { dao_id: 0u64, max: 100u32 });
		let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency {
			dao_id: 0u64,
			call: Box::new(set_max_members),
		});
		let hash = BlakeTwo256::hash_of(&do_as_agency);

		assert!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			10,
			Box::new(do_as_agency.clone())
		)
		.is_err());
		MaxProposals::<Test>::insert(0u64, 0);
		assert!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		)
		.is_err());
		MaxProposals::<Test>::insert(0u64, 100);
		assert_ok!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		));

		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, true));
		assert!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, true).is_err());
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, false));
		assert!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, false).is_err());
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, true));

		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(3u64), 0, hash, 0, true));

		assert_ok!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 0));
		assert_eq!(crate::Pallet::<Test>::max_members(0u64), 100);
		MaxMembers::<Test>::insert(0u64, 50);
		assert_ok!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		));

		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 1, false));
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(3u64), 0, hash, 1, false));
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(4u64), 0, hash, 1, false));
		assert_ok!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 1));
		assert_eq!(crate::Pallet::<Test>::max_members(0u64), 50);

		assert_ok!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		));
		assert!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		)
		.is_err());
		assert!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 2).is_err());
		Prime::<Test>::insert(0u64, ALICE);
		assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(ALICE), 0, hash, 2, false));
		assert!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 2).is_err());
		frame_system::Pallet::<Test>::set_block_number(100000);
		assert_ok!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 2));

		assert!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		)
		.is_ok());
		Prime::<Test>::insert(0u64, ALICE);
		frame_system::Pallet::<Test>::set_block_number(200000);
		assert_ok!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 3));

		assert!(crate::Pallet::<Test>::propose(
			Origin::signed(ALICE),
			0u64,
			2,
			Box::new(do_as_agency.clone())
		)
		.is_ok());

		let disapprove_proposal =
			Call::Agency(crate::Call::disapprove_proposal { dao_id: 0u64, proposal_hash: hash });
		assert_ok!(sudo::Pallet::<Test>::sudo(
			Origin::signed(ALICE),
			0u64,
			Box::new(disapprove_proposal),
		));
	});
}
