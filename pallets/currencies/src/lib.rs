// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungible,
		fungibles::{Inspect, Mutate, Transfer},
	},
	transactional,
};
use frame_system::pallet_prelude::*;

use frame_support::traits::fungibles::{metadata, Create};
use frame_system::Config as SystemConfig;
pub use pallet::*;
use pallet_support::{fungibles::AssetFronze, uniqueid::UniqueIdGenerator};
use sp_runtime::traits::{Saturating, StaticLookup};
use sp_std::vec::Vec;

mod impl_fungibles;

mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::MultiCurrency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
	<<T as Config>::MultiCurrency as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// who can create new asset
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		/// The native currency id
		#[pallet::constant]
		type NativeToken: Get<AssetIdOf<Self>>;

		/// pallet_assets
		type MultiCurrency: Inspect<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ Create<Self::AccountId>
			+ metadata::Inspect<Self::AccountId>
			+ metadata::Mutate<Self::AccountId>;

		/// native currency
		type NativeCurrency: fungible::Inspect<Self::AccountId, Balance = BalanceOf<Self>>
			+ fungible::Mutate<Self::AccountId, Balance = BalanceOf<Self>>
			+ fungible::Transfer<Self::AccountId, Balance = BalanceOf<Self>>;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ObjectId = AssetIdOf<Self>>;

		/// The asset id
		#[pallet::constant]
		type AssetId: Get<AssetIdOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// #[pallet::storage]
	// #[pallet::getter(fn frozen_balance_get)]
	// pub(super) type FrozenBalances<T: Config> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	T::AccountId,
	// 	Twox64Concat,
	// 	AssetIdOf<T>,
	// 	BalanceOf<T>,
	// 	OptionQuery,
	// >;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AssetIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(AssetIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Burned(AssetIdOf<T>, T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Value overflow.
		ValueOverflow,
		/// Insufficient account balance.
		InsufficientBalance,
		/// The asset does not exist.
		AssetNotExisted,
		/// The balance does not exist.
		BalanceNotEnough,
		/// The frozen balance does not exist.
		FrozenBalanceNotExist,
		/// The frozen balance does not enough.
		FrozenBalanceNotEnough,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue a new class of fungible assets from a public origin.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn create(
			origin: OriginFor<T>,
			admin: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] min_balance: BalanceOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			#[pallet::compact] decimals: u8,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin.clone())?;
			let admin = T::Lookup::lookup(admin)?;
			let index = T::UniqueId::generate_object_id(T::AssetId::get())?;
			T::MultiCurrency::create(index, admin.clone(), true, min_balance)?;
			<T::MultiCurrency as metadata::Mutate<_>>::set(index, &admin, name, symbol, decimals)
		}

		/// Transfer some balance to another account under `asset_id`.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			asset_id: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			keep_alive: bool,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;

			<Self as Transfer<_>>::transfer(asset_id, &from, &to, amount, keep_alive)?;
			Ok(())
		}

		/// Exactly as `transfer`, except the origin must be root and the source account may be
		/// specified.
		/// # <weight>
		/// - Same as transfer, but additional read and write because the source account is not
		///   assumed to be in the overlay.
		/// # </weight>
		#[pallet::weight(10_000)]
		pub fn force_transfer(
			origin: OriginFor<T>,
			source: <T::Lookup as StaticLookup>::Source,
			dest: <T::Lookup as StaticLookup>::Source,
			asset_id: AssetIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let from = T::Lookup::lookup(source)?;
			let to = T::Lookup::lookup(dest)?;

			<Self as Transfer<T::AccountId>>::transfer(asset_id, &from, &to, amount, false)?;
			Ok(())
		}

		/// Transfer some native currency to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(10_000)]
		pub fn transfer_native(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: BalanceOf<T>,
			keep_alive: bool,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as Transfer<T::AccountId>>::transfer(
				T::NativeToken::get(),
				&from,
				&to,
				amount,
				keep_alive,
			)?;
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
