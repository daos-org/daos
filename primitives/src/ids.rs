use super::*;
pub use codec::MaxEncodedLen;
use sp_runtime::traits::{CheckedAdd, One};
use sp_std::ops::{Add, Mul};

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct DaoId(pub u64);

impl From<DaoId> for u64 {
	fn from(x: DaoId) -> Self {
		x.0
	}
}

impl From<u64> for DaoId {
	fn from(x: u64) -> Self {
		DaoId(x)
	}
}

impl Mul<Self> for DaoId {
	type Output = DaoId;

	fn mul(self, rhs: Self) -> Self::Output {
		DaoId(self.0 * rhs.0)
	}
}

impl One for DaoId {
	fn one() -> Self {
		DaoId(1u64)
	}
}

impl Add<Self> for DaoId {
	type Output = DaoId;

	fn add(self, rhs: Self) -> Self::Output {
		DaoId(self.0.add(rhs.0))
	}
}

impl CheckedAdd for DaoId {
	fn checked_add(&self, v: &Self) -> Option<Self> {
		self.0.checked_add(v.0).map(DaoId)
	}
}
#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct Nft<ClassId>(pub ClassId);

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct Fungible<TokenId>(pub TokenId);

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct RoomId<Id>(pub Id);

impl<T: Encode + Decode, ClassId: Encode + Decode> AccountIdConversion<T> for Nft<ClassId> {
	fn into_account(&self) -> T {
		(b"nft ", self).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap()
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"nft " {
				return None
			}
			let mut cursor = &d[4..];
			let result = Decode::decode(&mut cursor).ok()?;
			if cursor.iter().all(|x| *x == 0) {
				Some(result)
			} else {
				None
			}
		})
	}
}

impl<T: Encode + Decode, TokenId: Encode + Decode> AccountIdConversion<T> for Fungible<TokenId> {
	fn into_account(&self) -> T {
		(b"fung", self).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap()
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"fung" {
				return None
			}
			let mut cursor = &d[4..];
			let result = Decode::decode(&mut cursor).ok()?;
			if cursor.iter().all(|x| *x == 0) {
				Some(result)
			} else {
				None
			}
		})
	}
}

impl<T: Encode + Decode, Id: Encode + Decode> AccountIdConversion<T> for RoomId<Id> {
	fn into_account(&self) -> T {
		(b"room", self).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap()
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"room" {
				return None
			}
			let mut cursor = &d[4..];
			let result = Decode::decode(&mut cursor).ok()?;
			if cursor.iter().all(|x| *x == 0) {
				Some(result)
			} else {
				None
			}
		})
	}
}
