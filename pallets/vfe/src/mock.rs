// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate as pallet_vfe;
use frame_support::{parameter_types, PalletId};
use frame_support_test::TestRandomness;
use frame_system as system;
use pallet_assets::FrozenBalance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};
use system::RawOrigin;

pub type AccountId = AccountId32;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const TOM: AccountId = AccountId::new([3u8; 32]);
pub const CANDY: AccountId = AccountId::new([4u8; 32]);
pub const DANY: AccountId = AccountId::new([5u8; 32]);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Instance = pallet_uniques::Instance1;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		VFEUniques: pallet_uniques::<Instance1>::{Pallet, Call, Storage, Event<T>},
		UniqueId: pallet_unique_id::{Pallet, Storage},
		Currencies: pallet_currencies::{Pallet, Call, Storage, Event<T>},
		VFE: pallet_vfe::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u64 = 0;
	pub const ApprovalDeposit: u64 = 0;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = TestFreezer;
	type WeightInfo = ();
	type Extra = ();
	type AssetAccountDeposit = ConstU64<0>;
}

use frame_support::traits::AsEnsureOriginWithArg;
use frame_system::EnsureSigned;
use sp_runtime::traits::ConstU64;
use std::{cell::RefCell, collections::HashMap};

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Hook {
	Died(u32, AccountId),
}
thread_local! {
	static FROZEN: RefCell<HashMap<(u32, AccountId), u64>> = RefCell::new(Default::default());
	static HOOKS: RefCell<Vec<Hook>> = RefCell::new(Default::default());
}

pub struct TestFreezer;

impl FrozenBalance<u32, AccountId, u64> for TestFreezer {
	fn frozen_balance(asset: u32, who: &AccountId) -> Option<u64> {
		FROZEN.with(|f| f.borrow().get(&(asset, who.clone())).cloned())
	}

	fn died(asset: u32, who: &AccountId) {
		HOOKS.with(|h| h.borrow_mut().push(Hook::Died(asset, who.clone())));
	}
}

pub struct EnsureBrand<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
	for EnsureBrand<AccountId>
{
	type Success = AccountId;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(who) if (who == CANDY) => Ok(who),
			r => Err(O::from(r)),
		})
	}
}

pub struct EnsureProducer<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
	for EnsureProducer<AccountId>
{
	type Success = AccountId;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(who) if (who == ALICE || who == TOM) => Ok(who),
			r => Err(O::from(r)),
		})
	}
}

parameter_types! {
	pub const ClassDeposit: u64 = 2;
	pub const InstanceDeposit: u64 = 1;
	pub const KeyLimit: u32 = 50;
	pub const ValueLimit: u32 = 50;
	pub const StringLimit: u32 = 500000;
	pub const MetadataDepositBase: u64 = 1;
	pub const AttributeDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_uniques::Config<Instance> for Test {
	type Event = Event;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CollectionDeposit = ClassDeposit;
	type ItemDeposit = InstanceDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = MetadataDepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
	type Locker = ();
}

impl pallet_unique_id::Config for Test {
	type ObjectId = u32;
	type StartId = ConstU32<1u32>;
	type MaxId = ConstU32<100u32>;
}

parameter_types! {
	pub const NativeToken: u32 = 0;
	pub const AssetId: u32 = u32::MAX - 1;
}

impl pallet_currencies::Config for Test {
	type Event = Event;
	type CreateOrigin = EnsureSigned<Self::AccountId>;
	type NativeToken = NativeToken;
	type MultiCurrency = Assets;
	type NativeCurrency = Balances;
	type UniqueId = UniqueId;
	type AssetId = AssetId;
}

parameter_types! {
	pub const VFEPalletId: PalletId = PalletId(*b"poc/acas");
	pub const ProducerId: u32 = u32::MAX - 2;
	pub const VFEBrandId: u32 = u32::MAX - 3;
	pub const IncentiveToken: u32 = 0;
	pub const UnbindFee:u32 = 1;
	pub const CostUnit: u64 = 100000;
	pub const EnergyRecoveryDuration: u64 = 8;
	pub const DailyEarnedResetDuration: u64 = 24;
	pub const LevelUpCostFactor: u64 = 7;
	pub const InitEnergy: u16 = 8;
	pub const InitEarningCap: u16 = 500;
	pub const EnergyRecoveryRatio: Permill = Permill::from_percent(25); //25%
}

impl Config for Test {
	type Event = Event;
	type BrandOrigin = EnsureBrand<Self::AccountId>;
	type ProducerOrigin = EnsureProducer<Self::AccountId>;
	type ProducerId = ProducerId;
	type VFEBrandId = VFEBrandId;
	type ObjectId = u32;
	type Currencies = Currencies;
	type PalletId = VFEPalletId;
	type UniqueId = UniqueId;
	type UniquesInstance = Instance;
	type Randomness = TestRandomness<Self>;
	type UnbindFee = UnbindFee;
	type CostUnit = CostUnit;
	type EnergyRecoveryDuration = EnergyRecoveryDuration;
	type DailyEarnedResetDuration = DailyEarnedResetDuration;
	type LevelUpCostFactor = LevelUpCostFactor;
	type InitEnergy = InitEnergy;
	type InitEarningCap = InitEarningCap;
	type EnergyRecoveryRatio = EnergyRecoveryRatio;

}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(ALICE, 10000000000), (BOB, 10000000000), (CANDY, 10000000000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_assets::GenesisConfig::<Test> {
		assets: vec![
			// id, owner, is_sufficient, min_balance
			(0, ALICE, true, 1),
			(1, ALICE, true, 1),
		],
		metadata: vec![
			// id, name, symbol, decimals
			(0, "PNT".into(), "PNT".into(), 12),
			(1, "FUN".into(), "FUN".into(), 12),
		],
		accounts: vec![
			// id, account_id, balance
			// (1, ALICE, 0),
			// (1, BOB, 0),
			// (1, TOM, 0),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub(crate) fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			VFE::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		VFE::on_initialize(System::block_number());
	}
}
