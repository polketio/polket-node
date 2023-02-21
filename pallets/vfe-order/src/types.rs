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
	pub collection_id: CollectionId,
	/// token id
	// #[codec(compact)]
	pub item_id: ItemId,

}


#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Order<AssetId,Balance, BlockNumber, BoundedItem> {
	/// currency ID.
	pub asset_id: AssetId,
	/// Price of this Instance.
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	pub deadline: BlockNumber,
	/// vfe list
	pub items: BoundedItem,

}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo,MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<AssetId,Balance, BlockNumber, OrderItem> {
	/// currency ID.
	pub asset_id: AssetId,
	/// Price of this Instance.
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	pub deadline: BlockNumber,
	/// vfe list
	pub item: OrderItem,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}