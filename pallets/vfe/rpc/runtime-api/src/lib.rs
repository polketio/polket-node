//! Runtime API definition for vfe module.

#![cfg_attr(not(feature = "std"), no_std)]
// The `too_many_arguments` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::too_many_arguments)]
// The `unnecessary_mut_passed` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
	pub trait VfeApi<AccountId, BrandId, ItemId, VFEDetail, Balance> where
		AccountId: Codec,
		BrandId: Codec,
		ItemId: Codec,
		VFEDetail: Codec,
		Balance: Codec,
	{
		fn get_vfe_details_by_address(account: AccountId, brand_id: BrandId) -> Vec<VFEDetail>;

	 	fn get_charging_costs(brand_id: BrandId, item: ItemId, charge_num: u16) -> Balance;

	 	fn get_level_up_costs(who: AccountId, brand_id: BrandId, item: ItemId) -> Balance;
	}

}
