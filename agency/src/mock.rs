#![allow(dead_code)]
#![allow(unused_imports)]
use crate as agency;
use frame_support::{
	debug, parameter_types,
	sp_tracing::debug,
	traits::{ConstU16, ConstU32, ConstU64, Contains},
};
use frame_system;
use primitives::{ids::Nft, traits::BaseCallFilter, types::MemberCount};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
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
		DAO: dao::{ Pallet, Call, Event<T>, Storage },
		Agency: agency::{ Pallet, Call, Event<T>, Storage, Origin<T> },
		Sudo: sudo::{ Pallet, Call, Event<T>, Storage },
		DoAs: daos_doas::{ Pallet, Call, Event<T>, Storage },
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
		match call {
			Call::DoAs(_) => {
				#[cfg(test)]
				println!("doas funcs");
				false
			},
			_ => {
				#[cfg(test)]
				println!("not doas funcs");
				true
			},
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

parameter_types! {
	pub const MaxMembersForSystem: MemberCount = 4;
}

pub struct BaseCall;

impl Contains<Call> for BaseCall {
	fn contains(_t: &Call) -> bool {
		true
	}
}

impl agency::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type Proposal = Call;
	type CollectiveBaseCallFilter = BaseCall;
	type DefaultVote = agency::PrimeDefaultVote;
	type MaxMembersForSystem = MaxMembersForSystem;
	type WeightInfo = ();
}

impl sudo::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

impl daos_doas::Config for Test {
	type Event = Event;
	type DoAsOrigin = ();
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = GenesisConfig { system: Default::default() }.build_storage().unwrap();
	t.into()
}
