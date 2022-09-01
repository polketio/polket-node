// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::HasCompact;
use frame_support::pallet_prelude::*;
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One};

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
		type CollectionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + HasCompact + MaxEncodedLen;
		/// The token ID type
		type ItemId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + HasCompact + MaxEncodedLen;
		/// The asset ID type
		type AssetId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + HasCompact;
		/// The asset ID type
		type NormalId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + HasCompact;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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
	type CollectionId = T::CollectionId;
	type ItemId = T::ItemId;
	type AssetId = T::AssetId;
	type NormalId = T::NormalId;

	/// generate new class id: Return the current ID, and increment the current ID
	fn generate_class_id() -> Result<Self::CollectionId, sp_runtime::DispatchError> {
		let class_id = NextCollectionId::<T>::try_mutate(|id| -> Result<T::CollectionId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::ValueOverflow)?;
			Ok(current_id)
		})?;
		Ok(class_id)
	}

	/// generate new instance id with class id: Return the current ID, and increment the current ID
	fn generate_instance_id(
		class_id: Self::CollectionId,
	) -> Result<Self::ItemId, sp_runtime::DispatchError> {
		//Check Is class_id generated?
		ensure!(class_id < Self::next_class_id(), Error::<T>::CollectionIdIsNotExisted);

		let instance_id = NextItemId::<T>::try_mutate(
			class_id,
			|id| -> Result<T::ItemId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::ValueOverflow)?;
				Ok(current_id)
			},
		)?;
		Ok(instance_id)
	}


	fn generate_asset_id() -> Result<Self::AssetId, sp_runtime::DispatchError> {
		let asset_id = NextAssetId::<T>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::ValueOverflow)?;
			Ok(current_id)
		})?;
		Ok(asset_id)
	}


	fn generate_normal_id() -> Result<Self::NormalId, sp_runtime::DispatchError> {
		let asset_id = NextNormalId::<T>::try_mutate(|id| -> Result<T::NormalId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::ValueOverflow)?;
			Ok(current_id)
		})?;
		Ok(asset_id)
	}
}
