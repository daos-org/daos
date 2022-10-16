#![allow(dead_code)]
use crate as sudo;
use frame_system;
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use primitives::ids::Nft;
use sp_std::result::Result;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
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
			_ => Ok(0u64)
		}
	}
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = GenesisConfig { system: Default::default() }.build_storage().unwrap();
	t.into()
}
