// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum UniqueStatusType {
	/// Unique Can be trade
	TradeEnable,
	/// Unique Can not be trade
	TradeUnEnable,
}

impl Default for UniqueStatusType {
	fn default() -> Self {
		UniqueStatusType::TradeUnEnable
	}
}



pub trait UniqueTradeGenerator {
	type AccountId;
	type CollectionId;
	type ItemId;

	/// check the collection_id and item_id from the account_id if it can be trade
	fn check_unique_trande(account_id: Self::AccountId,collection_id: Self::CollectionId,item_id: Self::ItemId) -> UniqueStatusType ;
}
