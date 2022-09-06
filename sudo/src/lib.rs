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

pub use dao::{self, BaseCallFilter};
pub use frame_support::{traits::UnfilteredDispatchable, weights::GetDispatchInfo};
pub use pallet::*;
use primitives::constant::weight::DAOS_BASE_WEIGHT;
pub use scale_info::{prelude::boxed::Box, TypeInfo};
pub use sp_std::{fmt::Debug, result};

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::Event::{CloseSudo, SetSudoAccount, SudoDone};
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use primitives::AccountIdConversion;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sudo_account)]
	pub type SudoAccount<T: Config> = StorageMap<_, Identity, T::DaoId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn is_must_democracy)]
	pub type IsMustDemocracy<T: Config> = StorageMap<_, Identity, T::DaoId, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SudoDone { sudo: T::AccountId, sudo_result: DispatchResult },
		SetSudoAccount { dao_id: T::DaoId, sudo_account: T::AccountId },
		CloseSudo { dao_id: T::DaoId, is_close: bool },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotSudo,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn sudo(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			call: Box<<T as dao::Config>::Call>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let sudo = Self::get_sudo_account(dao_id, IsMustDemocracy::<T>::get(dao_id))?;
			ensure!(sudo == who, Error::<T>::NotSudo);
			let concrete_id = dao::Pallet::<T>::try_get_concrete_id(dao_id)?;
			ensure!(concrete_id.contains(*call.clone()), dao::Error::<T>::InVailCall);
			// let _: T::CallId = TryFrom::<<T as dao::Config>::Call>::try_from(*call.clone())
			// 	.ok()
			// 	.ok_or(dao::Error::<T>::HaveNoCallId)?;

			let res = call.dispatch_bypass_filter(
				frame_system::RawOrigin::Signed(concrete_id.into_account()).into(),
			);
			Self::deposit_event(SudoDone {
				sudo,
				sudo_result: res.map(|_| ()).map_err(|e| e.error),
			});
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn set_sudo_account(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			sudo_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let sudo = Self::get_sudo_account(dao_id, IsMustDemocracy::<T>::get(dao_id))?;
			ensure!(sudo == who, Error::<T>::NotSudo);
			SudoAccount::<T>::insert(dao_id, sudo_account.clone());
			Self::deposit_event(SetSudoAccount { dao_id, sudo_account });
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn close_sudo(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			is_close: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let sudo = Self::get_sudo_account(dao_id, IsMustDemocracy::<T>::get(dao_id))?;
			ensure!(sudo == who, Error::<T>::NotSudo);
			IsMustDemocracy::<T>::insert(dao_id, is_close);
			Self::deposit_event(CloseSudo { dao_id, is_close });
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_sudo_account(
			dao_id: T::DaoId,
			is_must_democracy: bool,
		) -> result::Result<T::AccountId, DispatchError> {
			if is_must_democracy {
				Ok(dao::Pallet::<T>::try_get_concrete_id(dao_id)?.into_account())
			} else {
				Ok(SudoAccount::<T>::get(dao_id).unwrap_or(
					dao::Pallet::<T>::try_get_creator(dao_id)
						.unwrap_or(dao::Pallet::<T>::try_get_concrete_id(dao_id)?.into_account()),
				))
			}
		}
	}
}
