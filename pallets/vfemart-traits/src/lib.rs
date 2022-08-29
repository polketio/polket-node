#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]

pub use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect as MultiAssets, Transfer,Mutate as MultiAssetsMutate},
		tokens::nonfungibles::{Create, Inspect, Mutate},
	}
};

use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::PerU16;
use sp_std::{vec, vec::Vec};

pub mod constants_types;
pub use crate::constants_types::*;
pub use contract_types::*;
pub use log;

pub type ResultPost<T> = sp_std::result::Result<
	T,
	sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
>;

pub trait VfemartConfig<AccountId, BlockNumber> {
	fn auction_close_delay() -> BlockNumber;
	fn is_in_whitelist(_who: &AccountId) -> bool;
	fn get_min_order_deposit() -> Balance;
	fn get_then_inc_id() -> Result<GlobalId, DispatchError>;
	fn inc_count_in_category(category_id: GlobalId) -> DispatchResult;
	fn dec_count_in_category(category_id: GlobalId) -> DispatchResult;
	fn do_add_whitelist(who: &AccountId);
	fn do_create_category(metadata: VFEMetadata) -> DispatchResultWithPostInfo;
	fn peek_next_gid() -> GlobalId;
	fn get_royalties_rate() -> PerU16;
	fn get_platform_fee_rate() -> PerU16;
	fn get_max_commission_reward_rate() -> PerU16;
	fn get_min_commission_agent_deposit() -> Balance;
}

pub trait VfemartOrder<AccountId, ClassId, InstanceId> {
	fn burn_orders(owner: &AccountId, class_id: ClassId, instance_id: InstanceId) -> DispatchResult;
	fn burn_offers(owner: &AccountId, class_id: ClassId, instance_id: InstanceId) -> DispatchResult;
}

pub trait VfemartVfe<AccountId, ClassId, InstanceId> {
	fn peek_next_class_id() -> ClassId;
	fn transfer(
		from: &AccountId,
		to: &AccountId,
		class_id: ClassId,
		instance_id: InstanceId,
		quantity: InstanceId,
	) -> DispatchResult;
	fn account_token(
		_who: &AccountId,
		_class_id: ClassId,
		_token_id: InstanceId,
	) -> AccountToken<InstanceId>;
	fn reserve_tokens(
		who: &AccountId,
		class_id: ClassId,
		instance_id: InstanceId,
		quantity: InstanceId,
	) -> DispatchResult;
	fn unreserve_tokens(
		who: &AccountId,
		class_id: ClassId,
		instance_id: InstanceId,
		quantity: InstanceId,
	) -> DispatchResult;
	fn token_charged_royalty(
		class_id: ClassId,
		instance_id: InstanceId,
	) -> Result<(AccountId, PerU16), DispatchError>;
	fn create_class(
		who: &AccountId,
		metadata: VFEMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(AccountId, ClassId)>;
	fn update_class(
		who: &AccountId,
		class_id: ClassId,
		metadata: VFEMetadata,
		name: Vec<u8>,
		description: Vec<u8>,
		royalty_rate: PerU16,
		properties: Properties,
		category_ids: Vec<GlobalId>,
	) -> ResultPost<(AccountId, ClassId)>;
	fn proxy_mint(
		delegate: &AccountId,
		to: &AccountId,
		class_id: ClassId,
		metadata: VFEMetadata,
		quantity: InstanceId,
		charge_royalty: Option<PerU16>,
	) -> ResultPost<(AccountId, AccountId, ClassId, InstanceId, InstanceId)>;
}

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?))
	}
}
impl TypeInfo for Properties {
	type Identity = Self;
	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<ClassProperty>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("ClassProperty")))
	}
}




/// Account Token
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AccountToken<InstanceId> {
	/// account token number.
	#[codec(compact)]
	pub quantity: InstanceId,
	/// account reserved token number.
	#[codec(compact)]
	pub reserved: InstanceId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<BlockNumber> {
	/// The minimum balance to create class
	#[codec(compact)]
	pub deposit: Balance,
	/// Property of all tokens in this class.
	pub properties: Properties,
	/// Name of class.
	pub name: Vec<u8>,
	/// Description of class.
	pub description: Vec<u8>,
	#[codec(compact)]
	pub create_block: BlockNumber,
	#[codec(compact)]
	pub royalty_rate: PerU16,
	/// Category of this class.
	pub category_ids: Vec<GlobalId>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData<AccountId, BlockNumber> {
	/// The minimum balance to create token
	#[codec(compact)]
	pub deposit: Balance,
	#[codec(compact)]
	pub create_block: BlockNumber,
	/// Charge royalty
	#[codec(compact)]
	pub royalty_rate: PerU16,
	/// The token's creator
	pub creator: AccountId,
	/// Royalty beneficiary
	pub royalty_beneficiary: AccountId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CategoryData {
	/// The category metadata.
	pub metadata: VFEMetadata,
	/// The number of orders/auctions in this category.
	#[codec(compact)]
	pub count: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrderItem<ClassId, InstanceId> {
	/// class id
	#[codec(compact)]
	pub class_id: ClassId,
	/// token id
	#[codec(compact)]
	pub instance_id: InstanceId,
	/// quantity
	#[codec(compact)]
	pub quantity: InstanceId,
}

#[cfg(feature = "std")]
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default, TypeInfo,
)]
pub struct ClassConfig<ClassId, AccountId, InstanceId> {
	pub class_id: ClassId,
	pub class_metadata: String,
	pub category_ids: Vec<GlobalId>,
	pub name: String,
	pub description: String,
	pub royalty_rate: PerU16,
	pub properties: u8,
	pub admins: Vec<AccountId>,
	pub tokens: Vec<TokenConfig<AccountId, InstanceId>>,
}

#[cfg(feature = "std")]
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Serialize, Deserialize, Default, TypeInfo,
)]
pub struct TokenConfig<AccountId, InstanceId> {
	pub instance_id: InstanceId,
	pub token_metadata: String,
	pub royalty_rate: PerU16,
	pub token_owner: AccountId,
	pub token_creator: AccountId,
	pub royalty_beneficiary: AccountId,
	pub quantity: InstanceId,
}

/// Check only one royalty constrains.
pub fn count_charged_royalty<AccountId, ClassId, InstanceId, VFE>(
	items: &[(ClassId, InstanceId, InstanceId)],
) -> ResultPost<(u32, AccountId, PerU16)>
where
	VFE: VfemartVfe<AccountId, ClassId, InstanceId>,
	ClassId: Copy,
	InstanceId: Copy,
	AccountId: Default,
{
	let mut count_of_charged_royalty: u32 = 0;
	let mut royalty_rate = PerU16::zero();
	let mut who = AccountId::default();
	for (class_id, instance_id, _quantity) in items {
		let (id, rate) = VFE::token_charged_royalty(*class_id, *instance_id)?;
		if !rate.is_zero() {
			count_of_charged_royalty = count_of_charged_royalty.saturating_add(1u32);
			royalty_rate = rate;
			who = id;
		}
	}
	Ok((count_of_charged_royalty, who, royalty_rate))
}

/// Swap assets between vfes owner and vfes purchaser.
#[allow(clippy::too_many_arguments)]
pub fn swap_assets<MultiCurrency, VFE, AccountId, ClassId, InstanceId, AssetId>(
	pay_currency: &AccountId,
	pay_vfes: &AccountId,
	asset_id: AssetId,
	price: Balance,
	items: &[(ClassId, InstanceId, InstanceId)],
	treasury: &AccountId,
	platform_fee_rate: PerU16,
	beneficiary: &AccountId,
	royalty_rate: PerU16,
	commission_agent: &Option<(bool, AccountId, PerU16)>,
) -> ResultPost<()>
where
	MultiCurrency:MultiAssets<AccountId,AssetId = AssetId,Balance = Balance> + Transfer<AccountId,AssetId = AssetId,Balance = Balance> + MultiAssetsMutate<AccountId,AssetId = AssetId,Balance = Balance>,
	VFE: VfemartVfe<AccountId, ClassId, InstanceId>,
	ClassId: Copy,
	InstanceId: Copy,
	AssetId: Copy,
{
	let trading_fee = platform_fee_rate.mul_ceil(price);
	let royalty_fee = royalty_rate.mul_ceil(price);
	MultiCurrency::transfer(asset_id, pay_currency, pay_vfes, price,true)?;
	MultiCurrency::transfer(asset_id, pay_vfes, treasury, trading_fee,true)?;
	MultiCurrency::transfer(asset_id, pay_vfes, beneficiary, royalty_fee,true)?;
	if let Some((status, agent, rate)) = commission_agent {
		if *status {
			let r = price.saturating_sub(trading_fee).saturating_sub(royalty_fee);
			MultiCurrency::transfer(asset_id, pay_vfes, agent, rate.mul_ceil(r),true)?;
		}
	}

	for (class_id, instance_id, quantity) in items {
		VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
	}
	Ok(())
}

#[macro_export]
macro_rules! to_item_vec {
	($obj: ident, $commission_agent: ident) => {{
		let items = $obj.items.iter().map(|x| (x.class_id, x.instance_id, x.quantity)).collect::<Vec<(
			ClassIdOf<T>,
			InstanceIdOf<T>,
			InstanceIdOf<T>,
		)>>();

		let commission_agent: Option<(bool, T::AccountId, PerU16)> =
			$commission_agent.and_then(|ca| {
				let b: Balance = <T as Config>::Currency::total_balance(&ca).saturated_into();
				if b < T::ExtraConfig::get_min_commission_agent_deposit() ||
					$obj.commission_rate.is_zero()
				{
					Some((false, ca, $obj.commission_rate))
				} else {
					Some((true, ca, $obj.commission_rate))
				}
			});

		(items, commission_agent)
	}};
}

#[macro_export]
macro_rules! ensure_one_royalty {
	($items: ident) => {{
		let (c, id, r) =
			count_charged_royalty::<T::AccountId, ClassIdOf<T>, InstanceIdOf<T>, T::VFE>(&$items)?;
		ensure!(c <= 1, Error::<T>::TooManyTokenChargedRoyalty);
		(id, r)
	}};
}

#[macro_export]
macro_rules! vfe_dbg {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "vfemart", log::Level::Debug, $($msg),+);
	};
}

#[macro_export]
macro_rules! vfe_info {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "vfemart", log::Level::Info, $($msg),+);
	};
}

#[macro_export]
macro_rules! vfe_err {
	($($msg: expr),+ $(,)?) => {
		#[cfg(test)]
		println!($($msg),+);
		#[cfg(not(test))]
		log::log!(target: "vfemart", log::Level::Error, $($msg),+);
	};
}

pub fn reserve_and_push_tokens<AccountId, ClassId, InstanceId, VFE>(
	vfe_owner: Option<&AccountId>,
	items: &[(ClassId, InstanceId, InstanceId)],
	push_to: &mut Vec<OrderItem<ClassId, InstanceId>>,
) -> ResultPost<()>
where
	VFE: VfemartVfe<AccountId, ClassId, InstanceId>,
	ClassId: Copy,
	InstanceId: Copy,
{
	for &(class_id, instance_id, quantity) in items {
		if let Some(vfe_owner) = vfe_owner {
			VFE::reserve_tokens(vfe_owner, class_id, instance_id, quantity)?;
		}
		push_to.push(OrderItem { class_id, instance_id, quantity })
	}
	Ok(())
}
