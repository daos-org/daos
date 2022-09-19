use super::*;
use sp_runtime::DispatchError;

pub struct BadOrigin;

impl From<BadOrigin> for &'static str {
	fn from(_: BadOrigin) -> &'static str {
		"Bad origin"
	}
}

pub trait BaseCallFilter<Call> {
	fn contains(&self, call: Call) -> bool;
}

pub trait SetCollectiveMembers<AccountId: Clone + Ord, DaoId: Clone + Default + Copy, DispathErr> {
	fn set_members_sorted(
		dao_id: DaoId,
		members: &[AccountId],
		prime: Option<AccountId>,
	) -> result::Result<(), DispathErr>;
}

pub trait AfterCreate<A, B> {
	fn do_something(a: A, b: B);
}

impl<A: Clone, B: Clone> AfterCreate<A, B> for () {
	fn do_something(_a: A, _b: B) {}
}

pub trait TryCreate<AccountId: Clone + Ord, DaoId: Clone, DispatchError> {
	fn try_create(&self, who: AccountId, dao_id: DaoId) -> result::Result<(), DispatchError>;
}

impl<AccountId: Clone + Ord, DaoId: Clone + Default + Copy>
	SetCollectiveMembers<AccountId, DaoId, DispatchError> for ()
{
	fn set_members_sorted(
		_dao_id: DaoId,
		_members: &[AccountId],
		_prime: Option<AccountId>,
	) -> result::Result<(), DispatchError> {
		Ok(())
	}
}

/// Some sort of check on the origin is performed by this object.
pub trait EnsureOriginWithArg<OuterOrigin, Argument> {
	/// A return type.
	type Success;

	/// Perform the origin check.
	fn ensure_origin(o: OuterOrigin, a: &Argument) -> Result<Self::Success, BadOrigin> {
		Self::try_origin(o, a).map_err(|_| BadOrigin)
	}

	/// Perform the origin check, returning the origin value if unsuccessful. This allows chaining.
	fn try_origin(o: OuterOrigin, a: &Argument) -> Result<Self::Success, OuterOrigin>;

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(a: &Argument) -> OuterOrigin;
}


impl<AccountId: Clone + Ord, DaoId: Default + Clone, Id: Clone> TryCreate<AccountId, DaoId, DispatchError> for ids::Nft<Id> {
	fn try_create(&self, _who: AccountId, _dao_id: DaoId) -> Result<(), DispatchError> {
		Ok(())
	}
}

impl<Call: Clone, Id: Default> BaseCallFilter<Call> for ids::Nft<Id> {
	fn contains(&self, _call: Call) -> bool {
		true
	}
}
