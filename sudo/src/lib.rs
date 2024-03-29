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

//! # Sudo Module
//!
//! ## Module Introduction
//! Sudo module can execute all external transactions that DAO can execute.
//!
//! This module, very useful in the early stage of DAO creation, can be used to set basic parameters,
//! but it also means centralization. So to achieve true decentralization should `close_sudo`.

pub use dao::{self, BaseCallFilter};
pub use frame_support::{traits::UnfilteredDispatchable};
pub use pallet::*;
pub use scale_info::{prelude::boxed::Box, TypeInfo};
pub use sp_std::{fmt::Debug, result};
use weights::WeightInfo;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::Event::{CloseSudo, SetSudo, SudoDone};
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Root account id.
	#[pallet::storage]
	#[pallet::getter(fn sudo_account)]
	pub type Account<T: Config> = StorageMap<_, Identity, T::DaoId, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// root executes external transaction successfully.
		SudoDone { sudo: T::AccountId, sudo_result: DispatchResult },
		/// Set root account or reopen sudo.
		SetSudo { dao_id: T::DaoId, sudo_account: T::AccountId },
		/// delete root account.
		CloseSudo { dao_id: T::DaoId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Not a sudo account, nor a dao account.
		NotSudo,
		RootNotExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Execute external transactions as root
		#[pallet::weight(<T as pallet::Config>::WeightInfo::sudo())]
		pub fn sudo(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			call: Box<<T as dao::Config>::Call>,
		) -> DispatchResultWithPostInfo {
			let sudo = Self::check_origin(dao_id, origin)?;
			let concrete_id = dao::Pallet::<T>::try_get_concrete_id(dao_id)?;
			ensure!(concrete_id.contains(*call.clone()), dao::Error::<T>::InVailCall);

			let res = call.dispatch_bypass_filter(
				frame_system::RawOrigin::Signed(dao::Pallet::<T>::try_get_dao_account_id(dao_id)?)
					.into(),
			);
			Self::deposit_event(SudoDone {
				sudo,
				sudo_result: res.map(|_| ()).map_err(|e| e.error),
			});
			Ok(().into())
		}

		/// call id: 401
		///
		/// Set root account or reopen sudo.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_sudo_account())]
		pub fn set_sudo_account(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			sudo_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let _sudo = Self::check_origin(dao_id, origin)?;
			Account::<T>::insert(dao_id, sudo_account.clone());
			Self::deposit_event(SetSudo { dao_id, sudo_account });
			Ok(().into())
		}

		/// call id: 402
		///
		/// delete root account.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::close_sudo())]
		pub fn close_sudo(origin: OriginFor<T>, dao_id: T::DaoId) -> DispatchResultWithPostInfo {
			let _sudo = Self::check_origin(dao_id, origin)?;
			Account::<T>::take(dao_id);

			Self::deposit_event(CloseSudo { dao_id });
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn check_origin(
			dao_id: T::DaoId,
			o: OriginFor<T>,
		) -> result::Result<T::AccountId, DispatchError> {
			if let Ok(who) = dao::Pallet::<T>::ensrue_dao_root(o.clone(), dao_id) {
				Ok(who)
			} else {
				let who = ensure_signed(o)?;
				ensure!(
					who == Account::<T>::get(dao_id).ok_or(Error::<T>::RootNotExists)?,
					Error::<T>::NotSudo
				);
				Ok(who)
			}
		}
	}
}
