// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::HasCompact;
use frame_support::pallet_prelude::*;
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use sp_runtime::{traits::{AtLeast32BitUnsigned, CheckedAdd, One, Zero, Bounded}, TypeId, SaturatedConversion};
use sp_runtime::traits::{CheckedDiv, Saturating};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The Object ID type
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// the start value of id
		#[pallet::constant]
		type StartId: Get<Self::ObjectId>;

		/// the max value of id
		#[pallet::constant]
		type MaxId: Get<Self::ObjectId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Next available object ID.
	#[pallet::storage]
	#[pallet::getter(fn next_object_id)]
	pub type NextObjectId<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::ObjectId,
		T::ObjectId,
		ValueQuery,
	>;


	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Value overflow.
		ValueOverflow,
		/// Class id is not existed
		CollectionIdIsNotExisted,
	}
}

impl<T: Config> UniqueIdGenerator for Pallet<T> {
	type ObjectId = T::ObjectId;

	/// generate new object id: Return the current ID, and increment the current ID
	fn generate_object_id(parentId: Self::ObjectId) -> Result<Self::ObjectId, sp_runtime::DispatchError> {
		let asset_id = NextObjectId::<T>::try_mutate(parentId, |id| -> Result<T::ObjectId, DispatchError> {
			if id.is_zero() {
				*id = T::StartId::get();
			}
			let current_id = *id;
			ensure!(current_id <= T::MaxId::get(), Error::<T>::ValueOverflow);
			*id = id.saturating_add(One::one());;
			Ok(current_id)
		})?;
		Ok(asset_id)
	}
}
