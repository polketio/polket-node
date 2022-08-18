// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Implementations for fungibles trait.

use super::*;

impl<T: Config>  pallet_assets::FrozenBalance<T::AssetId, T::AccountId, T::Balance> for  Pallet<T>  {
	fn frozen_balance(asset: T::AssetId, who: &T::AccountId) -> Option<T::Balance> {
		if let Some(frozen) = FrozenBalances::<T>::get(who,asset){
			return Some(frozen);
		}
		None
	}

	fn died(_asset: T::AssetId, _who: &T::AccountId) {}
}


impl<T: Config>  AssetFronze<T::AssetId, T::AccountId, T::Balance> for Pallet<T>  {
	fn frozen_balance(
		from: &T::AccountId,
		asset_id: T::AssetId,
		value: T::Balance,
	) ->  Result<(), sp_runtime::DispatchError>{
		
		// if let Some(frozen) = FrozenBalances::<T>::try_mutate_exists(from, asset_id,  |maybe_order| {

		// }else{

		// }
		if  FrozenBalances::<T>::try_mutate_exists(from, asset_id, |maybe_order| {

				

		}).is_err(){
			
		};
		Ok(())
	}


	fn unfrozen_balance(
		from: &T::AccountId,
		asset_id: T::AssetId,
		value: T::Balance,
	) ->  Result<(), sp_runtime::DispatchError>{
		Ok(())
	}

}