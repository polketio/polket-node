// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
use codec::HasCompact;
use frame_support::pallet_prelude::*;
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use sp_runtime::{traits::{AtLeast32BitUnsigned, CheckedAdd, One, Zero}, TypeId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The class ID type
		type CollectionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The token ID type
		type ItemId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The asset ID type
		type AssetId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The asset ID type
		type NormalId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The Object ID type
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
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

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextCollectionId<T: Config> = StorageValue<_, T::CollectionId, ValueQuery>;

	/// Next available instance ID.
	#[pallet::storage]
	#[pallet::getter(fn next_instance_id)]
	pub type NextItemId<T: Config> =
	StorageMap<_, Twox64Concat, T::CollectionId, T::ItemId, ValueQuery>;


	/// Next available asset ID.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	/// Next available normal ID.
	#[pallet::storage]
	#[pallet::getter(fn next_normal_id)]
	pub type NextNormalId<T: Config> = StorageValue<_, T::NormalId, ValueQuery>;

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
				*id = One::one();
			}
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::ValueOverflow)?;
			Ok(current_id)
		})?;
		Ok(asset_id)
	}
}
