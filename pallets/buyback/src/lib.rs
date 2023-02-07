// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect as MultiAssets, Mutate as MultiAssetsMutate, Transfer},
		tokens::nonfungibles::{
			Create, Inspect, InspectEnumerable, Mutate, Transfer as NFTTransfer,
		},
		Randomness, UnixTime,
	},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use num_integer::Roots;
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::prelude::*;

// #[cfg(test)]
// mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		///  The origin which who can create buyback plan.
		type BuybackOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// Multiple Asset hander, which should implement `frame_support::traits::fungibles`
		type Currencies: MultiAssets<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ MultiAssetsMutate<Self::AccountId>;

		/// Unify the value types of AssetId
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ParentId = Self::Hash, ObjectId = Self::ObjectId>;

		/// The buyback plan-id parent key
		#[pallet::constant]
		type PlanId: Get<Self::Hash>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u128, u128, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// calculate
		#[pallet::weight(10_000)]
		pub fn calculate(origin: OriginFor<T>, num: u128) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let num_pow = num.pow(10);
			let num_nth: u128 = num_pow.nth_root(9);
			Self::deposit_event(Event::SomethingStored(num_pow, num_nth, who));
			Ok(())
		}
	}
}
