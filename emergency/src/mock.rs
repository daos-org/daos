#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate as emergency;
use frame_support::{
	debug, parameter_types,
	sp_tracing::debug,
	traits::{ConstU16, ConstU32, ConstU64, Contains},
};
use frame_system::{self, Account, EnsureRoot};
use primitives::{ids::Nft, traits::BaseCallFilter, types::MemberCount};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, DispatchError,
};
use sp_std::result::Result;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{

		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Emergency: emergency::{ Pallet, Call, Event<T>, Storage },
		DAO: dao::{ Pallet, Call, Event<T>, Storage },
		Sudo: sudo::{ Pallet, Call, Event<T>, Storage },
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl TryFrom<Call> for u64 {
	type Error = ();
	fn try_from(call: Call) -> Result<Self, Self::Error> {
		match call {
			_ => Ok(0u64),
		}
	}
}
impl BaseCallFilter<Call> for Nft<u64> {
	fn contains(&self, call: Call) -> bool {
		true
	}
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

impl dao::Config for Test {
	type Event = Event;
	type Call = Call;
	type CallId = u64;
	type DaoId = u64;
	type ConcreteId = Nft<u64>;
	type AfterCreate = ();
	type WeightInfo = ();
}

impl sudo::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinPledge: u64 = 100u64;
	pub const TrackPeriod: u64 = 100u64;
}

impl emergency::Config for Test {
	type Event = Event;
	type ExternalOrigin = EnsureRoot<u64>;
	type Currency = Balances;
	type MinPledge = MinPledge;
	type TrackPeriod = TrackPeriod;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 10), (3, 10), (10, 100), (20, 100), (30, 100)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
