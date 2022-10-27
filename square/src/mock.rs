#![allow(dead_code)]
use crate as square;
use crate::Pledge;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{ConstU16, ConstU32, ConstU64},
	RuntimeDebug,
};
use frame_system;
use primitives::ids::Nft;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DispatchError,
};
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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DAO: dao::{ Pallet, Call, Event<T>, Storage },
		Square: square::{ Pallet, Call, Event<T>, Storage },
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

#[derive(
	PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen, Default,
)]
pub struct Vote(pub u64);

impl Pledge<u64, u64, u64, (), u64, DispatchError> for Vote {
	fn try_vote(
		&self,
		_who: &u64,
		_dao_id: &u64,
		_conviction: &(),
	) -> Result<(u64, u64), DispatchError> {
		Ok((100u64, 100u64))
	}

	fn vote_end_do(&self, _who: &u64, _dao_id: &u64) -> Result<(), DispatchError> {
		Ok(())
	}
}

impl square::Config for Test {
	type Event = Event;
	type Pledge = Vote;
	type Conviction = ();
	type Currency = Balances;
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
