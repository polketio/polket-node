// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PlanStatus {
	/// Waiting for startup
	Upcoming = 0,
	/// Plan is in progress, sellers can lock asset in it.
	InProgress = 1,
	/// Plan is Completed, sellers can withdraw rewards.
	Completed = 2,
	/// All rewards has been paybacked.
	AllPaybacked = 3,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum BuybackMode {
	/// Burn
	Burn = 0,
	/// Transfer
	Transfer = 1,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PlanInfo<AccountId, AssetId, Balance, BlockNumber> {
	pub sell_asset_id: AssetId,
	pub buy_asset_id: AssetId,
	pub status: PlanStatus,
	pub min_sell: Balance,
	pub start: BlockNumber,
	pub period: BlockNumber,
	pub total_sell: Balance,
	pub total_buy: Balance,
	pub seller_amount: u32,
	pub seller_limit: u32,
	pub creator: AccountId,
	pub mode: BuybackMode,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ParticipantInfo<Balance> {
	pub locked: Balance,
	pub rewards: Balance,
	pub withdrew: bool,
}

impl<Balance: Default> Default for ParticipantInfo<Balance> {
	fn default() -> Self {
		ParticipantInfo { locked: Balance::default(), rewards: Balance::default(), withdrew: false }
	}
}
