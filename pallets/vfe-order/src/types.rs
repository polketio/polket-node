// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::TypeInfo, RuntimeDebug};
use sp_runtime::{traits::Get, BoundedVec,PerU16};
use sp_std::vec::Vec;



/// GlobalId ID type.
pub type GlobalId = u64;


#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrderItem<CollectionId, ItemId> {
	/// class id
	// #[codec(compact)]
	pub class_id: CollectionId,
	/// token id
	// #[codec(compact)]
	pub instance_id: ItemId,
	/// quantity
	// #[codec(compact)]
	pub quantity: ItemId,
}


#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(StringLimit))]
pub struct Order<AssetId,Balance, BlockNumber,CollectionId, ItemId,StringLimit: Get<u32>> {
	/// currency ID.
	pub asset_id: AssetId,
	/// The balances to create an order
	pub deposit: Balance,
	/// Price of this Instance.
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	pub deadline: BlockNumber,
	/// vfe list
	pub items: BoundedVec<OrderItem<CollectionId, ItemId>, StringLimit>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<AssetId,Balance, BlockNumber> {
	/// currency ID.
	pub asset_id: AssetId,
	/// Price of this Instance.
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	pub deadline: BlockNumber,
	/// vfe list
	// pub items: Vec<OrderItem<CollectionId, ItemId>>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}