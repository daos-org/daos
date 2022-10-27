// Copyright 2022 daos-org.
// This file is part of DAOS

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]
#![allow(clippy::wrong_self_convention)]
pub use codec::MaxEncodedLen;
pub use frame_support::codec::{Decode, Encode};
pub use scale_info::TypeInfo;
pub use sp_runtime::RuntimeDebug;
pub use sp_std::{prelude::*, result, vec};
pub mod constant;
pub mod ids;
pub mod traits;
pub mod types;

/// This type can be converted into and possibly from an [`AccountId`] (which itself is generic).
pub trait AccountIdConversion<AccountId>: Sized {
	/// Convert into an account ID. This is infallible.
	fn into_account(&self) -> AccountId;

	/// Try to convert an account ID into this type. Might not succeed.
	fn try_from_account(a: &AccountId) -> Option<Self>;
}

pub struct TrailingZeroInput<'a>(pub &'a [u8]);
impl<'a> codec::Input for TrailingZeroInput<'a> {
	fn remaining_len(&mut self) -> Result<Option<usize>, codec::Error> {
		Ok(None)
	}

	fn read(&mut self, into: &mut [u8]) -> Result<(), codec::Error> {
		let len = into.len().min(self.0.len());
		into[..len].copy_from_slice(&self.0[..len]);
		for i in &mut into[len..] {
			*i = 0;
		}
		self.0 = &self.0[len..];
		Ok(())
	}
}
