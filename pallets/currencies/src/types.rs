// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Implementations for fungibles trait.

use super::*;


pub struct Freezer<T>(PhantomData<T>);

impl <T: Config>  pallet_assets::FrozenBalance<AssetIdOf<T>, T::AccountId, BalanceOf<T>> for  Freezer<T> {
	fn frozen_balance(asset: AssetIdOf<T>, who: &T::AccountId) -> Option< BalanceOf<T>> {
        if let Some(frozen) = FrozenBalances::<T>::get(who,asset){
			return Some(frozen);
		}
		None
	}

	fn died(_asset: AssetIdOf<T>, _who: &T::AccountId) {

	}
}