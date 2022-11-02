use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::TypeInfo;
use frame_support::RuntimeDebug;
use sp_runtime::BoundedVec;
use sp_runtime::traits::Get;

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
			SportType::JumpRope => 120,    //120 jumps/minute
			SportType::Run => 10,
			SportType::Bicycle => 30,
		}
	}

	pub fn is_frequency_range(&self, frequency: u16) -> u16 {
		match self {
			SportType::JumpRope => {
				if 80 <= frequency && frequency <= 400 {
					1
				} else {
					0
				}
			}
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
	pub pk: [u8; 33],
	pub nonce: u32,
	pub timestamp: u32,
	pub mint_cost: Option<(AssetId, Balance)>,
}

#[derive(Encode, Decode, Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct VFEAbility {
	pub efficiency: u16,
	pub skill: u16,
	pub luck: u16,
	pub durable: u16,
}

#[derive(Encode, Decode, Copy, Clone, Default, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
