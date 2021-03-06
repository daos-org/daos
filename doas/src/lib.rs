// Copyright 2022 LISTEN TEAM.
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

pub use codec::MaxEncodedLen;
use dao::{self, BaseDaoCallFilter};
pub use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	pallet_prelude::StorageDoubleMap,
	Parameter,
};
pub use pallet::*;
use primitives::constant::weight::DAOS_BASE_WEIGHT;
pub use primitives::{
	traits::EnsureOriginWithArg,
	types::{DoAsEnsure, MemberCount, Proportion, RealCallId},
	AccountIdConversion,
};
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
	use frame_support::{
		dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::UnfilteredDispatchable,
	};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type DoAsOrigin: EnsureOriginWithArg<
			Self::Origin,
			(Self::DaoId, Self::CallId),
			Success = Self::DaoId,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetEnsure(T::DaoId, u32, DoAsEnsure<Proportion<MemberCount>, MemberCount>),
		DoAsDone { sudo_result: DispatchResult },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		DaoNotExists,
		NotDaoAccount,
		CallNotSupport,
		InVailDaoId,
		InVailCall,
		InVailCallId,
		HaveNoCallId,
		BadOrigin,
		ProportionErr,
		NotDaoId,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn do_as_collective(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			call: Box<<T as dao::Config>::Call>,
		) -> DispatchResultWithPostInfo {
			ensure!(
				dao::Pallet::<T>::get_second_id(dao_id)?.contains(*call.clone()),
				dao::Error::<T>::NotDaoSupportCall
			);
			let call_id: T::CallId = TryFrom::<<T as dao::Config>::Call>::try_from(*call.clone())
				.ok()
				.ok_or(Error::<T>::HaveNoCallId)?;

			let id = T::DoAsOrigin::try_origin(origin, &(dao_id, call_id))
				.map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(dao_id == id, dao::Error::<T>::NotDaoId);
			let second_id = dao::Pallet::<T>::get_second_id(dao_id)?;
			let dao_account = second_id.into_account();
			let res =
				call.dispatch_bypass_filter(frame_system::RawOrigin::Signed(dao_account).into());
			Self::deposit_event(Event::DoAsDone {
				sudo_result: res.map(|_| ()).map_err(|e| e.error),
			});
			Ok(().into())
		}
	}
}
