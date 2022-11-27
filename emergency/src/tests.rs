use super::*;
use crate::mock::{Call, Origin, *};
use frame_support::assert_ok;
use primitives::ids::Nft;
use sp_runtime::traits::BlakeTwo256;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;

pub fn create_dao() {
	dao::Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

fn rec_balance() {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 10000u64);
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(&BOB, 10000u64);
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
	assert_eq!(sudo::Account::<Test>::get(0u64), Some(ALICE));
}

pub fn set_members() {
	set_sudo();
	let proposal_1 = Call::Emergency(crate::Call::set_members { dao_id: 0u64, members: vec![ALICE,] });
	assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(proposal_1)));
	assert!(crate::Members::<Test>::get(0u64).len() > 0);
}

fn get_proposal() -> Vec<u8> {
	Call::Emergency(
		crate::Call::set_pledge { dao_id: 0u64, amount: 1000u64 }).encode()
}

fn external() {
	rec_balance();
	let proposal = Call::decode(&mut &get_proposal()[..]).unwrap();
	assert_ok!(crate::Pallet::<Test>::external_track(
	Origin::root(),
	0u64,
	Box::new(proposal),
	vec![1, 2, 3, 4]
));
	assert!(crate::HashesOf::<Test>::get(0u64).len() > 0);

}

fn internal() {
	set_members();
	rec_balance();
	let proposal = Call::decode(&mut &get_proposal()[..]).unwrap();
	assert_ok!(crate::Pallet::<Test>::internal_track(
		Origin::signed(ALICE),
		0u64,
		Box::new(proposal.clone()),
		vec![1, 2, 3, 4],
	));
	assert!(crate::Pallet::<Test>::internal_track(
		Origin::signed(ALICE),
		0u64,
		Box::new(proposal),
		vec![1, 2, 3, 4]
	).is_err());
	assert!(crate::HashesOf::<Test>::get(0u64).len() > 0);
	assert!(crate::HashesOf::<Test>::get(0u64).contains(&BlakeTwo256::hash(&get_proposal()[..])));
	assert!(crate::ProposalOf::<Test>::contains_key(0u64, BlakeTwo256::hash(&get_proposal()[..])));
}

#[test]
fn set_xx_should_work() {
	new_test_ext().execute_with(|| {
		set_members();
		let proposal = Call::decode(&mut &get_proposal()[..]).unwrap();
		assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(proposal)));
		assert_eq!(crate::PledgeOf::<Test>::get(0u64), 1000u64);
	})
}


#[test]
fn external_track_should_work() {
	new_test_ext().execute_with(|| {
		external();
	});
}

#[test]
fn internal_track_should_work() {
	new_test_ext().execute_with(||{
		internal();
	});
}

#[test]
fn reject_internal_track() {
	new_test_ext().execute_with(|| {
		internal();
		frame_system::Pallet::<Test>::set_block_number(10000);
		assert!(crate::ProposalOf::<Test>::contains_key(0u64, BlakeTwo256::hash(&get_proposal()[..])));
		assert!(crate::Pallet::<Test>::reject(
			Origin::signed(ALICE),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		).is_err());
		frame_system::Pallet::<Test>::set_block_number(0);
		assert_ok!(Pallet::<Test>::reject(
			Origin::signed(ALICE),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		));
		assert_eq!(ProposalOf::<Test>::contains_key(0u64, BlakeTwo256::hash(&get_proposal()[..])), false)
	});
}

#[test]
fn reject_external_track() {
	new_test_ext().execute_with(|| {
		external();
		assert!(crate::Pallet::<Test>::reject(
			Origin::signed(ALICE),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		).is_err());
		set_members();
		assert_ok!(
			crate::Pallet::<Test>::reject(
			Origin::signed(ALICE),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		)
		);
		assert_eq!(ProposalOf::<Test>::contains_key(0u64, BlakeTwo256::hash(&get_proposal()[..])), false)

	});
}

#[test]
fn enact_proposal_should_work() {
	new_test_ext().execute_with(|| {
		internal();
		assert_ne!(crate::PledgeOf::<Test>::get(0u64), 1000);
		assert!(crate::Pallet::<Test>::enact_proposal(
			Origin::signed(BOB),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		).is_err());
		assert!(crate::Pallet::<Test>::enact_proposal(
			Origin::signed(BOB),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..2]),
		).is_err());
		frame_system::Pallet::<Test>::set_block_number(10000);
		assert_ok!(crate::Pallet::<Test>::enact_proposal(
			Origin::signed(BOB),
			0u64,
			BlakeTwo256::hash(&get_proposal()[..]),
		));
		assert_eq!(crate::PledgeOf::<Test>::get(0u64), 1000);
	});
}