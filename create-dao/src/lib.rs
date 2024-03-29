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
#![allow(clippy::type_complexity)]

//! # Create DAO Module
//!
//! ## Module Introduction
//! Create the DAO for a specific group.
//!
//! Users are free to create their own DAOs according to the requirements of the DAO template.
//!
//! Therefore, developers should create a DAO template in advance, which determines which users are members of the DAO
//! and which transactions can be executed by this type of DAO.
//!
//! ## How to create a DAO template
//! As a developer, if you don't know how to create a DAO template, you can refer to the code linked below, this is a vault DAO template of the kico project.
//! * [Vault Module](https://github.com/DICO-TEAM/dico-chain/blob/main/pallets/vc/src/lib.rs)
//! * [How to use daos for Vault Module](https://github.com/DICO-TEAM/dico-chain/blob/main/runtime/tico/src/vc.rs)

pub use codec::{MaxEncodedLen, Decode, Encode};
pub use frame_support::{
	// codec::{Decode, Encode},
	traits::IsSubType,
};
use frame_support::{sp_runtime::traits::BlockNumberProvider, };
pub use pallet::*;
pub use primitives::{
	traits::{AfterCreate, BaseCallFilter, TryCreate},
	types::RealCallId,
	AccountIdConversion,
};
pub use scale_info::{prelude::boxed::Box, TypeInfo};
// use sp_runtime::traits::BlockNumberProvider;
pub use sp_runtime::{traits::Hash, RuntimeDebug};
pub use sp_std::{
	marker::PhantomData,
	prelude::{self, *},
	result,
};
use weights::WeightInfo;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
/// DAO's status.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum Status {
	/// In use.
	Active,
	/// Does not work properly.
	InActive,
}

/// DAO specific information
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DaoInfo<AccountId, BlockNumber, ConcreteId, Status> {
	/// creator of DAO.
	creator: AccountId,
	/// The block that creates the DAO.
	pub start_block: BlockNumber,
	/// The id of the specific group mapped by dao.
	pub concrete_id: ConcreteId,
	/// DAO account id.
	pub dao_account_id: AccountId,
	/// Description of the DAO.
	// describe: Vec<u8>,
	/// State of the DAO.
	status: Status,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::UnfilteredDispatchable,
		// weights::GetDispatchInfo,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedAdd, One};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// All calls supported by DAO.
		type Call: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			// + GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ From<Call<Self>>
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Each Call has its own id.
		type CallId: Parameter
			+ Copy
			+ MaybeSerializeDeserialize
			+ TypeInfo
			+ MaxEncodedLen
			+ Default
			+ TryFrom<<Self as pallet::Config>::Call>;

		/// Each DAO has its own id.
		type DaoId: Clone + Default + Copy + Parameter + Member + MaxEncodedLen + CheckedAdd + One;

		/// The specific group on the chain mapped by DAO.
		type ConcreteId: Parameter
			+ Member
			+ TypeInfo
			+ MaxEncodedLen
			+ Clone
			+ Copy
			+ Default
			+ AccountIdConversion<Self::AccountId>
			+ BaseCallFilter<<Self as pallet::Config>::Call>
			+ TryCreate<Self::AccountId, Self::DaoId, DispatchError>;

		/// Do some things after creating dao, such as setting up a sudo account.
		type AfterCreate: AfterCreate<Self::AccountId, Self::DaoId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// #[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// All DAOs that have been created.
	#[pallet::storage]
	#[pallet::getter(fn daos)]
	pub type Daos<T: Config> = StorageMap<
		_,
		Identity,
		T::DaoId,
		DaoInfo<T::AccountId, u32, T::ConcreteId, Status>,
	>;

	/// The id of the next dao to be created.
	#[pallet::storage]
	#[pallet::getter(fn next_dao_id)]
	pub type NextDaoId<T: Config> = StorageValue<_, T::DaoId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The new DAO is successfully created.
		CreatedDao(T::AccountId, T::DaoId, T::ConcreteId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Do not have permission to create.
		HaveNoCreatePermission,
		/// DAO already exists
		DaoExists,
		/// DAO does not exist.
		DaoNotExists,
		/// DAO unsupported call
		InVailCall,
		/// Wrong origin.
		BadOrigin,
		/// Not the id of this dao.
		DaoIdNotMatch,
		/// The description of the DAO is too long.
		DescribeTooLong,
		/// Numerical calculation overflow error.
		Overflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a DAO for a specific group
		#[pallet::weight(T::WeightInfo::create_dao())]
		pub fn create_dao(
			origin: OriginFor<T>,
			concrete_id: T::ConcreteId,
			describe: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			ensure!(describe.len() <= 50, Error::<T>::DescribeTooLong);
			let dao_id = NextDaoId::<T>::get();

			if !cfg!(any(feature = "std", feature = "runtime-benchmarks")) {
				concrete_id.try_create(creator.clone(), dao_id)?;
			}

			// fixme
			let now = frame_system::Pallet::<T>::current_block_number();

			Daos::<T>::insert(
				dao_id,
				DaoInfo {
					creator: creator.clone(),
					start_block: 100,
					concrete_id,
					// describe,
					status: Status::Active,
					dao_account_id: concrete_id.into_account(),
				},
			);
			let next_id = dao_id.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
			NextDaoId::<T>::put(next_id);
			T::AfterCreate::do_something(creator.clone(), dao_id);
			Self::deposit_event(Event::CreatedDao(creator, dao_id, concrete_id));
			Ok(().into())
		}

		/// call id:101
		///
		/// dao remark something.
		#[pallet::weight(T::WeightInfo::dao_remark())]
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
		) -> Result<DaoInfo<T::AccountId, u32, T::ConcreteId, Status>, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao)
		}

		pub fn try_get_concrete_id(
			dao_id: <T as pallet::Config>::DaoId,
		) -> result::Result<T::ConcreteId, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao.concrete_id)
		}

		pub fn try_get_dao_account_id(
			dao_id: <T as pallet::Config>::DaoId,
		) -> result::Result<T::AccountId, DispatchError> {
			let dao = Daos::<T>::get(dao_id).ok_or(Error::<T>::DaoNotExists)?;
			Ok(dao.dao_account_id)
		}

		pub fn ensrue_dao_root(
			o: OriginFor<T>,
			dao_id: T::DaoId,
		) -> result::Result<T::AccountId, DispatchError> {
			let who = ensure_signed(o)?;
			let dao_id = Self::try_get_dao_account_id(dao_id)?;
			ensure!(who == dao_id, Error::<T>::BadOrigin);
			Ok(who)
		}
	}
}
