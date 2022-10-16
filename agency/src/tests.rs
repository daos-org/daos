#![allow(unused_imports)]
#![cfg(test)]

use super::*;
use sudo;
use crate::mock::{Call, Origin, *};
use frame_support::{assert_noop, assert_ok, debug};
use primitives::ids::Nft;
use sp_runtime::traits::BlakeTwo256;

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
    let proposal = Call::Sudo(sudo::Call::set_sudo_account {
        dao_id: 0u64,
        sudo_account: ALICE,
    });
    sudo::Pallet::<Test>::sudo(Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, Box::new(proposal)).unwrap();
    assert_eq!(sudo::Account::<Test>::get(0u64), Some(1u64));
}

fn set_origin_for_0() {
    set_sudo();
    let proposal = Call::Agency(crate::Call::set_ensure_origin_for_every_call{
        dao_id: 0u64,
        call_id: 0u64,
        ensure: DoAsEnsureOrigin::Member,
    });
    assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, Box::new(proposal)));
}

fn set_origin_for_0_1() {
    set_sudo();
    let proposal = Call::Agency(crate::Call::set_ensure_origin_for_every_call{
        dao_id: 0u64,
        call_id: 0u64,
        ensure: DoAsEnsureOrigin::Members(2u32),
    });
    assert_ok!(sudo::Pallet::<Test>::sudo(Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, Box::new(proposal)));
}

#[test]
fn set_origin_should_work() {
    new_test_ext().execute_with(|| {
        set_origin_for_0();
    });
}

#[test]
fn execute() {
    new_test_ext().execute_with(|| {
        set_origin_for_0();
        let set_max_members = Call::Agency(crate::Call::set_max_members {
            dao_id: 0u64,
            max: 100u32,
        });

        let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency{
            dao_id: 0u64,
            call: Box::new(set_max_members),
        });
        assert_ok!(crate::Pallet::<Test>::execute(Origin::signed(ALICE), 0u64, Box::new(do_as_agency)));

        assert_eq!(crate::Pallet::<Test>::max_members(0u64), 100);
    });
}

#[test]
fn proposal_vote_close_should_work() {
    new_test_ext().execute_with(||{
        set_origin_for_0_1();
        let set_max_members = Call::Agency(crate::Call::set_max_members {
            dao_id: 0u64,
            max: 100u32,
        });
        let do_as_agency = Call::DoAs(daos_doas::Call::do_as_agency{
            dao_id: 0u64,
            call: Box::new(set_max_members),
        });
        let hash = BlakeTwo256::hash_of(&do_as_agency);

        assert_ok!(crate::Pallet::<Test>::propose(Origin::signed(ALICE), 0u64, 2, Box::new(do_as_agency)));
        assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(2u64), 0, hash, 0, true));
        assert_ok!(crate::Pallet::<Test>::vote(Origin::signed(3u64), 0, hash, 0, true));
        assert_ok!(crate::Pallet::<Test>::close(Origin::signed(4u64), 0, hash, 0));
        assert_eq!(crate::Pallet::<Test>::max_members(0u64), 100);
    });
}

