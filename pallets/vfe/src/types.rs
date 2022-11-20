// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::TypeInfo, RuntimeDebug};
use sp_runtime::{traits::Get, BoundedVec};
use sp_std::vec::Vec;

/// public key of device
pub type DeviceKey = [u8; 33];

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DeviceStatus {
	/// Registered
	Registered = 0,
	/// Activated
	Activated = 1,
	/// Voided
	Voided = 2,
}

impl Default for DeviceStatus {
	fn default() -> Self {
		DeviceStatus::Registered
	}
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SportType {
	/// JumpRope
	JumpRope = 0,
	/// Run
	Run = 1,
	/// Bicycle
	Bicycle = 2,
}

impl Default for SportType {
	fn default() -> Self {
		SportType::JumpRope
	}
}

impl SportType {
	pub fn training_unit_duration(&self) -> u16 {
		match self {
			SportType::JumpRope => 30,
			SportType::Run => 60,
			SportType::Bicycle => 60,
		}
	}

	pub fn frequency_standard(&self) -> u16 {
		match self {
			SportType::JumpRope => 120, //120 jumps/minute
			SportType::Run => 10,
			SportType::Bicycle => 30,
		}
	}

	pub fn is_frequency_range(&self, frequency: u16) -> u16 {
		match self {
			SportType::JumpRope =>
				if 80 <= frequency && frequency <= 400 {
					1
				} else {
					0
				},
			SportType::Run => 1,
			SportType::Bicycle => 1,
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
pub struct User<Account, BlockNumber> {
	pub owner: Account,
	pub energy_total: u16,
	pub energy: u16,
	pub create_block: BlockNumber,
	pub last_restore_block: BlockNumber,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Producer<ObjectId, Account> {
	pub id: ObjectId,
	pub owner: Account,
}

#[derive(Eq, PartialEq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
pub struct VFEBrand<CollectionId, StringLimit: Get<u32>> {
	pub brand_id: CollectionId,
	pub sport_type: SportType,
	pub rarity: VFERarity,
	pub approvals: u32,
	pub uri: BoundedVec<u8, StringLimit>,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VFEBrandApprove<AssetId, Balance> {
	pub mint_cost: Option<(AssetId, Balance)>,
	pub remaining_mint: u32,
	pub activated: u32,
	pub registered: u32,
	pub locked_of_mint: Balance,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Device<Class, Instance, ObjectId, AssetId, Balance> {
	pub sport_type: SportType,
	pub brand_id: Class,
	pub item_id: Option<Instance>,
	pub producer_id: ObjectId,
	pub status: DeviceStatus,
	pub pk: DeviceKey,
	pub nonce: u32,
	pub timestamp: u32,
	pub mint_cost: Option<(AssetId, Balance)>,
}

#[derive(
	Encode, Decode, Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct VFEAbility {
	pub efficiency: u16,
	pub skill: u16,
	pub luck: u16,
	pub durable: u16,
}

#[derive(
	Encode, Decode, Copy, Clone, Default, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct VFEDetail<Class, Instance, Hash, BlockNumber> {
	pub class_id: Class,
	pub instance_id: Instance,
	pub base_ability: VFEAbility,
	pub current_ability: VFEAbility,
	pub rarity: VFERarity,
	pub level: u16,
	pub remaining_battery: u16,
	pub gene: Hash,
	pub is_upgrading: bool,
	pub last_block: BlockNumber,
	pub available_points: u16,
}

#[derive(Encode, Decode, Default, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Item<Class, Instance> {
	pub class_id: Class,
	pub instance_id: Instance,
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum VFERarity {
	/// Common
	Common = 0,
	/// Elite
	Elite = 1,
	/// Rare
	Rare = 2,
	/// Epic
	Epic = 3,
}

impl Default for VFERarity {
	fn default() -> Self {
		VFERarity::Common
	}
}

impl VFERarity {
	pub fn base_range_of_ability(&self) -> (u16, u16) {
		match self {
			VFERarity::Common => (2, 8),
			VFERarity::Elite => (6, 12),
			VFERarity::Rare => (10, 18),
			VFERarity::Epic => (20, 30),
		}
	}

	pub fn growth_points(&self) -> u16 {
		match self {
			VFERarity::Common => 4,
			VFERarity::Elite => 4,
			VFERarity::Rare => 4,
			VFERarity::Epic => 4,
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct JumpRopeTrainingReport {
	pub timestamp: u32,
	pub training_duration: u16,
	pub total_jump_rope_count: u16,
	pub average_speed: u16,
	pub max_speed: u16,
	pub max_jump_rope_count: u16,
	pub interruptions: u8,
	pub jump_rope_duration: u16,
}

impl TryFrom<Vec<u8>> for JumpRopeTrainingReport {
	type Error = ();

	fn try_from(report_data: Vec<u8>) -> Result<Self, Self::Error> {
		if report_data.len() < 17 {
			return Result::Err(())
		}
		let timestamp_vec = report_data[0..4].try_into().map_err(|_| ())?;
		let skipping_times_vec: [u8; 2] = report_data[4..6].try_into().map_err(|_| ())?;
		let training_count_vec: [u8; 2] = report_data[6..8].try_into().map_err(|_| ())?;
		let average_frequency_vec: [u8; 2] = report_data[8..10].try_into().map_err(|_| ())?;
		let maximum_frequency_vec: [u8; 2] = report_data[10..12].try_into().map_err(|_| ())?;
		let maximum_skipping_vec: [u8; 2] = report_data[12..14].try_into().map_err(|_| ())?;
		let number_of_miss = report_data[14];
		let jump_rope_duration_vec: [u8; 2] = report_data[15..17].try_into().map_err(|_| ())?;

		Ok(JumpRopeTrainingReport {
			timestamp: u32::from_le_bytes(timestamp_vec),
			training_duration: u16::from_le_bytes(skipping_times_vec),
			total_jump_rope_count: u16::from_le_bytes(training_count_vec),
			average_speed: u16::from_le_bytes(average_frequency_vec),
			max_speed: u16::from_le_bytes(maximum_frequency_vec),
			max_jump_rope_count: u16::from_le_bytes(maximum_skipping_vec),
			interruptions: number_of_miss,
			jump_rope_duration: u16::from_le_bytes(jump_rope_duration_vec),
		})
	}
}

impl Into<Vec<u8>> for JumpRopeTrainingReport {
	fn into(self) -> Vec<u8> {
		let mut bytes: Vec<u8> = Vec::new();
		bytes.extend(self.timestamp.to_le_bytes());
		bytes.extend(self.training_duration.to_le_bytes());
		bytes.extend(self.total_jump_rope_count.to_le_bytes());
		bytes.extend(self.average_speed.to_le_bytes());
		bytes.extend(self.max_speed.to_le_bytes());
		bytes.extend(self.max_jump_rope_count.to_le_bytes());
		bytes.extend(self.interruptions.to_le_bytes());
		bytes.extend(self.jump_rope_duration.to_le_bytes());
		bytes
	}
}
