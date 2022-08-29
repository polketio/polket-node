// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{fungible, fungibles},
};
use frame_system::pallet_prelude::*;

use frame_system::Config as SystemConfig;
pub use pallet::*;
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero,AtLeast32BitUnsigned},
};
use codec::HasCompact;
mod impl_fungibles;
use scale_info::TypeInfo;
use pallet_support::fungibles::AssetFronze;



#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct FrozenBalance<Account, Balance> {
	owner: Account,
	frozen_balance: Balance,
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Identifier for the class of asset.
		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn frozen_balance_get)]
	pub(super) type FrozenBalances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		T::AssetId,
		T::Balance,
		OptionQuery,
	>;



	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// AccountBalanceNotExist
		AccountBalanceNotExist,

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {


	}


}
