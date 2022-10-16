#![allow(unused_imports)]
#![cfg(test)]

use super::*;
use crate::mock::{Call, *};
use frame_support::{assert_noop, assert_ok, debug};
use primitives::ids::Nft;

pub const ALICE: u64 = 1;

pub fn create_dao() {
    dao::Pallet::<Test>::create_dao(Origin::signed(ALICE), Nft(0u64), vec![1; 4]).unwrap();
}

pub fn set_sudo() {
    create_dao();
    let proposal = Call::Sudo(crate::Call::set_sudo_account {
        dao_id: 0u64,
        sudo_account: ALICE,
    });
    crate::Pallet::<Test>::sudo(Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, Box::new(proposal));
    assert_eq!(crate::Account::<Test>::get(0u64), Some(1u64));
}

#[test]
pub fn sudo_should_work() {
    new_test_ext().execute_with(|| {
        create_dao();
        let proposal = Call::DAO(dao::Call::dao_remark {
            dao_id: 0u64,
            remark: vec![1; 10],
        });
        assert_ok!(crate::Pallet::<Test>::sudo(Origin::signed(dao::Daos::<Test>::get(0u64).unwrap().dao_account_id), 0u64, Box::new(proposal)));
    });
}

#[test]
pub fn close_sudo_should_work() {
    new_test_ext().execute_with(|| {
        set_sudo();
        let proposal = Call::Sudo(crate::Call::close_sudo {
            dao_id: 0u64,
        });
        crate::Pallet::<Test>::sudo(Origin::signed(ALICE), 0u64, Box::new(proposal));
        assert_eq!(crate::Account::<Test>::get(0u64), None);
    });
}
