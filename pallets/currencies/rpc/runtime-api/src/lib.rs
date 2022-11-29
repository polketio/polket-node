//! Runtime API definition for vfe module.

#![cfg_attr(not(feature = "std"), no_std)]
// The `too_many_arguments` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::too_many_arguments)]
// The `unnecessary_mut_passed` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;

sp_api::decl_runtime_apis! {
	pub trait CurrenciesApi<AccountId, AssetId, Balance> where
		AccountId: Codec,
		AssetId: Codec,
		Balance: Codec,
	{
		/// The total amount of issuance in the system.
		fn total_issuance(asset: AssetId) -> Balance;

		/// The minimum balance any single account may have.
		fn minimum_balance(asset: AssetId) -> Balance;

		/// Get the `asset` balance of `who`.
		fn balance(asset: AssetId, who: AccountId) -> Balance;

		/// Get the maximum amount of `asset` that `who` can withdraw/transfer successfully.
		fn reducible_balance(asset: AssetId, who: AccountId, keep_alive: bool) -> Balance;
	}
}
