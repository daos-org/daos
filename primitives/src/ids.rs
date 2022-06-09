use super::*;
pub use codec::MaxEncodedLen;

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

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct Nft<ClassId>(pub ClassId);

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct Fungible<TokenId>(pub TokenId);

#[derive(Decode, Encode, Copy, Clone, Default, Debug, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub struct RoomId<Id>(pub Id);

// #[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
// pub enum SecondId<ClassId, TokenId> {
// 	Nft(ClassId),
// 	Fungible(TokenId)
// }
//
// impl<T: Encode + Decode, ClassId: Encode + Decode, TokenId: Encode + Decode> AccountIdConversion<T> for SecondId<ClassId, TokenId> {
// 	fn into_account(&self) -> T {
// 		match self {
// 			SecondId::Nft(x) => (b"nft ", Nft(x)).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap(),
// 			SecondId::Fungible(x) => (b"fung", Fungible(x)).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap(),
// 		}
// 	}
//
// 	fn try_from_account(x: &T) -> Option<Self> {
// 		x.using_encoded(|d| {
// 			if &d[0..4] != b"nft " && &d[0..4] != b"fung"{
// 				return None
// 			}
// 			let mut cursor = &d[4..];
// 			let result = Decode::decode(&mut cursor).ok()?;
// 			if cursor.iter().all(|x| *x == 0) {
// 				Some(result)
// 			} else {
// 				None
// 			}
// 		})
// 	}
// }

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

impl<T: Encode + Decode> AccountIdConversion<T> for DaoId {
	fn into_account(&self) -> T {
		(b"dao ", self).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap()
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"dao " {
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

