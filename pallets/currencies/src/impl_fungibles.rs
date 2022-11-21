// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Implementations for fungibles trait.

use super::*;
use frame_support::{
	sp_std::borrow::ToOwned,
	traits::{
		fungible, fungibles,
		tokens::{DepositConsequence, WithdrawConsequence},
	},
};
use sp_std::{vec, vec::Vec};

impl<T: Config> fungibles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	type AssetId = AssetIdOf<T>;
	type Balance = BalanceOf<T>;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		// Asset::<T, I>::get(asset).map(|x| x.supply).unwrap_or_else(Zero::zero)
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::total_issuance()
		} else {
			T::MultiCurrency::total_issuance(asset)
		}
	}

	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		// Asset::<T, I>::get(asset).map(|x| x.min_balance).unwrap_or_else(Zero::zero)
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::minimum_balance()
		} else {
			T::MultiCurrency::minimum_balance(asset)
		}
	}

	fn balance(asset: Self::AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
		// Pallet::<T, I>::balance(asset, who)
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::balance(who)
		} else {
			T::MultiCurrency::balance(asset, who)
		}
	}

	fn reducible_balance(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		keep_alive: bool,
	) -> Self::Balance {
		// Pallet::<T, I>::reducible_balance(asset, who, keep_alive).unwrap_or(Zero::zero())
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::reducible_balance(who, keep_alive)
		} else {
			T::MultiCurrency::reducible_balance(asset, who, keep_alive)
		}
	}

	fn can_deposit(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
		mint: bool,
	) -> DepositConsequence {
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::can_deposit(who, amount, mint)
		} else {
			T::MultiCurrency::can_deposit(asset, who, amount, mint)
		}
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Inspect<_>>::can_withdraw(who, amount)
		} else {
			T::MultiCurrency::can_withdraw(asset, who, amount)
		}
	}
}

impl<T: Config> fungibles::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn mint_into(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Mutate<_>>::mint_into(who, amount)?
		} else {
			T::MultiCurrency::mint_into(asset, who, amount)?
		}
		Self::deposit_event(Event::Issued(asset, who.to_owned(), amount));
		Ok(())
	}

	fn burn_from(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		let actual = if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Mutate<_>>::burn_from(who, amount)?
		} else {
			T::MultiCurrency::burn_from(asset, who, amount)?
		};
		Self::deposit_event(Event::Burned(asset, who.to_owned(), actual));
		Ok(actual)
	}

	fn slash(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Mutate<_>>::slash(who, amount)
		} else {
			T::MultiCurrency::slash(asset, who, amount)
		}
	}
}

impl<T: Config> fungibles::Transfer<T::AccountId> for Pallet<T> {
	fn transfer(
		asset: Self::AssetId,
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: Self::Balance,
		keep_alive: bool,
	) -> Result<Self::Balance, DispatchError> {
		let actual = if asset == T::NativeToken::get() {
			<T::NativeCurrency as fungible::Transfer<_>>::transfer(
				source, dest, amount, keep_alive,
			)?
		} else {
			T::MultiCurrency::transfer(asset, source, dest, amount, keep_alive)?
		};
		Self::deposit_event(Event::Transferred(asset, source.to_owned(), dest.to_owned(), actual));
		Ok(actual)
	}
}

impl<T: Config> fungibles::metadata::Inspect<T::AccountId> for Pallet<T> {
	fn name(asset: Self::AssetId) -> Vec<u8> {
		let actual =
			if asset == T::NativeToken::get() { vec![] } else { T::MultiCurrency::name(asset) };
		actual
	}

	fn symbol(asset: Self::AssetId) -> Vec<u8> {
		let actual =
			if asset == T::NativeToken::get() { vec![] } else { T::MultiCurrency::symbol(asset) };
		actual
	}

	fn decimals(asset: Self::AssetId) -> u8 {
		let actual =
			if asset == T::NativeToken::get() { 0 } else { T::MultiCurrency::decimals(asset) };
		actual
	}
}

impl<T: Config> AssetFronze<AssetIdOf<T>, T::AccountId, BalanceOf<T>> for Pallet<T> {
	fn frozen_balance(
		from: &T::AccountId,
		asset_id: AssetIdOf<T>,
		value: BalanceOf<T>,
	) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		let result = T::MultiCurrency::can_withdraw(asset_id, from, value);

		ensure!(result == WithdrawConsequence::Success, Error::<T>::BalanceNotEnough);

		FrozenBalances::<T>::try_mutate(
			from,
			asset_id,
			|maybe_balance| -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
				if let Some(f_balance) = maybe_balance {
					*maybe_balance = Some(f_balance.saturating_add(value));
				} else {
					*maybe_balance = Some(value);
				}
				Ok(value)
			},
		)
	}

	fn unfrozen_balance(
		from: &T::AccountId,
		asset_id: AssetIdOf<T>,
		value: BalanceOf<T>,
	) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		FrozenBalances::<T>::try_mutate(
			from,
			asset_id,
			|maybe_balance| -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
				let frozen = maybe_balance.ok_or(Error::<T>::FrozenBalanceNotExist)?;

				ensure!(frozen > value, Error::<T>::BalanceNotEnough);

				*maybe_balance = Some(frozen.saturating_sub(value));

				Ok(value)
			},
		)
	}
}

// impl<T: Config<I>, I: 'static> fungibles::Unbalanced<T::AccountId> for Pallet<T, I> {
// 	fn set_balance(_: Self::AssetId, _: &T::AccountId, _: Self::Balance) -> DispatchResult {
// 		unreachable!("set_balance is not used if other functions are impl'd");
// 	}
// 	fn set_total_issuance(id: T::AssetId, amount: Self::Balance) {
// 		Asset::<T, I>::mutate_exists(id, |maybe_asset| {
// 			if let Some(ref mut asset) = maybe_asset {
// 				asset.supply = amount
// 			}
// 		});
// 	}
// 	fn decrease_balance(
// 		asset: T::AssetId,
// 		who: &T::AccountId,
// 		amount: Self::Balance,
// 	) -> Result<Self::Balance, DispatchError> {
// 		let f = DebitFlags { keep_alive: false, best_effort: false };
// 		Self::decrease_balance(asset, who, amount, f, |_, _| Ok(()))
// 	}
// 	fn decrease_balance_at_most(
// 		asset: T::AssetId,
// 		who: &T::AccountId,
// 		amount: Self::Balance,
// 	) -> Self::Balance {
// 		let f = DebitFlags { keep_alive: false, best_effort: true };
// 		Self::decrease_balance(asset, who, amount, f, |_, _| Ok(())).unwrap_or(Zero::zero())
// 	}
// 	fn increase_balance(
// 		asset: T::AssetId,
// 		who: &T::AccountId,
// 		amount: Self::Balance,
// 	) -> Result<Self::Balance, DispatchError> {
// 		Self::increase_balance(asset, who, amount, |_| Ok(()))?;
// 		Ok(amount)
// 	}
// 	fn increase_balance_at_most(
// 		asset: T::AssetId,
// 		who: &T::AccountId,
// 		amount: Self::Balance,
// 	) -> Self::Balance {
// 		match Self::increase_balance(asset, who, amount, |_| Ok(())) {
// 			Ok(()) => amount,
// 			Err(_) => Zero::zero(),
// 		}
// 	}
// }

// impl<T: Config<I>, I: 'static> fungibles::metadata::Mutate<<T as SystemConfig>::AccountId>
// 	for Pallet<T, I>
// {
// 	fn set(
// 		asset: T::AssetId,
// 		from: &<T as SystemConfig>::AccountId,
// 		name: Vec<u8>,
// 		symbol: Vec<u8>,
// 		decimals: u8,
// 	) -> DispatchResult {
// 		Self::do_set_metadata(asset, from, name, symbol, decimals)
// 	}
// }

// impl<T: Config<I>, I: 'static> fungibles::approvals::Inspect<<T as SystemConfig>::AccountId>
// 	for Pallet<T, I>
// {
// 	// Check the amount approved to be spent by an owner to a delegate
// 	fn allowance(
// 		asset: T::AssetId,
// 		owner: &<T as SystemConfig>::AccountId,
// 		delegate: &<T as SystemConfig>::AccountId,
// 	) -> T::Balance {
// 		Approvals::<T, I>::get((asset, &owner, &delegate))
// 			.map(|x| x.amount)
// 			.unwrap_or_else(Zero::zero)
// 	}
// }

// impl<T: Config<I>, I: 'static> fungibles::approvals::Mutate<<T as SystemConfig>::AccountId>
// 	for Pallet<T, I>
// {
// 	fn approve(
// 		asset: T::AssetId,
// 		owner: &<T as SystemConfig>::AccountId,
// 		delegate: &<T as SystemConfig>::AccountId,
// 		amount: T::Balance,
// 	) -> DispatchResult {
// 		Self::do_approve_transfer(asset, owner, delegate, amount)
// 	}

// 	// Aprove spending tokens from a given account
// 	fn transfer_from(
// 		asset: T::AssetId,
// 		owner: &<T as SystemConfig>::AccountId,
// 		delegate: &<T as SystemConfig>::AccountId,
// 		dest: &<T as SystemConfig>::AccountId,
// 		amount: T::Balance,
// 	) -> DispatchResult {
// 		Self::do_transfer_approved(asset, owner, delegate, dest, amount)
// 	}
// }
