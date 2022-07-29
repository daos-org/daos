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
pub use frame_support::{
	codec::{Decode, Encode},
	traits::IsSubType,
};
pub use pallet::*;
pub use primitives::{
	constant::weight::DAOS_BASE_WEIGHT,
	traits::{BaseDaoCallFilter, GetCollectiveMembers, TryCreate},
	types::RealCallId,
	AccountIdConversion,
};
pub use scale_info::{prelude::boxed::Box, TypeInfo};
use sp_runtime::traits::BlockNumberProvider;
pub use sp_runtime::{traits::Hash, RuntimeDebug};
pub use sp_std::{
	marker::PhantomData,
	prelude::{self, *},
	result,
};
// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DaoInfo<AccountId, BlockNumber, ConcreteId> {
	creator: AccountId,
	pub start_block: BlockNumber,
	id: ConcreteId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::UnfilteredDispatchable,
		weights::GetDispatchInfo,
	};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Call: Parameter
			+ UnfilteredDispatchable<Origin = Self::Origin>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ From<Call<Self>>
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::Call>;

		type CallId: Parameter
			+ Copy
			+ MaybeSerializeDeserialize
			+ TypeInfo
			+ MaxEncodedLen
			+ Default
			+ TryFrom<<Self as pallet::Config>::Call>;

		type DaoId: Clone + Default + Copy + Parameter + Member + MaxEncodedLen;

		type ConcreteId: Parameter
			+ Member
			+ TypeInfo
			+ MaxEncodedLen
			+ Clone
			+ Default
			+ AccountIdConversion<Self::AccountId>
			+ BaseDaoCallFilter<<Self as pallet::Config>::Call>
			+ TryCreate<Self::AccountId, Self::DaoId, DispatchError>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn daos)]
	pub type Daos<T: Config> =
		StorageMap<_, Identity, T::DaoId, DaoInfo<T::AccountId, T::BlockNumber, T::ConcreteId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
		CreatedDao(T::AccountId, T::DaoId, T::ConcreteId),
	}

	#[pallet::error]
	pub enum Error<T> {
		HaveNoCreatePermission,
		DaoExists,
		DaoNotExists,
		InVailCallId,
		InVailCall,
		InVailDaoId,
		BadOrigin,
		NotDaoSupportCall,
		NotDaoId,
		HaveNoCallId,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn create_dao(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			concrete_id: T::ConcreteId,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			ensure!(!Daos::<T>::contains_key(dao_id), Error::<T>::DaoExists);

			if !cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
				concrete_id
					.try_create(creator.clone(), dao_id)
					.map_err(|_| Error::<T>::HaveNoCreatePermission)?;
			}

			let now = frame_system::Pallet::<T>::current_block_number();
			Daos::<T>::insert(
				dao_id,
				DaoInfo { creator: creator.clone(), start_block: now, id: concrete_id.clone() },
			);
			Self::deposit_event(Event::CreatedDao(creator, dao_id, concrete_id));
			Ok(().into())
		}

		#[pallet::weight(DAOS_BASE_WEIGHT)]
		pub fn dao_remark(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			_remark: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Self::ensrue_dao_root(origin, dao_id)?;
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn try_get_creator(
			dao_id: <T as pallet::Config>::DaoId,
		) -> result::Result<T::AccountId, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao.creator)
		}

		pub fn try_get_dao(
			dao_id: <T as pallet::Config>::DaoId,
		) -> result::Result<DaoInfo<T::AccountId, T::BlockNumber, T::ConcreteId>, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao)
		}

		pub fn try_get_concrete_id(
			dao_id: <T as pallet::Config>::DaoId,
		) -> result::Result<T::ConcreteId, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao.id)
		}

		pub fn ensrue_dao_root(
			o: OriginFor<T>,
			dao_id: T::DaoId,
		) -> Result<T::AccountId, DispatchError> {
			let who = ensure_signed(o)?;
			let concrete_id = Self::try_get_concrete_id(dao_id)?;
			ensure!(who == concrete_id.into_account(), Error::<T>::BadOrigin);
			Ok(who)
		}
	}
}
