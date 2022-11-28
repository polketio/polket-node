// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use sp_runtime::DispatchResult;

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type ItemId = T::ItemId;
	type CollectionId = T::CollectionId;

	fn owner(collection: &Self::CollectionId, instance: &Self::ItemId) -> Option<T::AccountId> {
		<pallet_uniques::Pallet<T, T::UniquesInstance> as Inspect<T::AccountId>>::owner(
			collection, instance,
		)
	}

	fn collection_owner(collection: &Self::CollectionId) -> Option<T::AccountId> {
		<pallet_uniques::Pallet<T, T::UniquesInstance> as Inspect<T::AccountId>>::collection_owner(
			collection,
		)
	}

	fn attribute(
		class: &Self::CollectionId,
		instance: &Self::ItemId,
		key: &[u8],
	) -> Option<Vec<u8>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::attribute(class, instance, key)
	}

	fn collection_attribute(collection: &Self::CollectionId, key: &[u8]) -> Option<Vec<u8>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::collection_attribute(collection, key)
	}

	fn can_transfer(class: &Self::CollectionId, instance: &Self::ItemId) -> bool {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::can_transfer(class, instance)
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T>
where
	T::CollectionId: From<T::ObjectId>,
	T::ItemId: From<T::ObjectId>,
	T::ObjectId: From<T::CollectionId>,
{
	fn mint_into(
		class: &Self::CollectionId,
		instance: &Self::ItemId,
		who: &T::AccountId,
	) -> DispatchResult {
		Self::do_mint(class.to_owned(), instance.to_owned(), who.to_owned())
	}

	fn burn(
		collection: &Self::CollectionId,
		instance: &Self::ItemId,
		_maybe_check_owner: Option<&T::AccountId>,
	) -> DispatchResult {
		Self::do_burn(collection.to_owned(), instance.to_owned())
	}
}

impl<T: Config> InspectEnumerable<T::AccountId> for Pallet<T> {
	/// Returns an iterator of the asset classes in existence.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn collections() -> Box<dyn Iterator<Item = Self::CollectionId>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::collections()
	}

	/// Returns an iterator of the instances of an asset `class` in existence.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn items(class: &Self::CollectionId) -> Box<dyn Iterator<Item = Self::ItemId>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::items(class)
	}

	/// Returns an iterator of the asset instances of all classes owned by `who`.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn owned(who: &T::AccountId) -> Box<dyn Iterator<Item = (Self::CollectionId, Self::ItemId)>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::owned(who)
	}

	/// Returns an iterator of the asset instances of `class` owned by `who`.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn owned_in_collection(
		class: &Self::CollectionId,
		who: &T::AccountId,
	) -> Box<dyn Iterator<Item = Self::ItemId>> {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::owned_in_collection(class, who)
	}
}
