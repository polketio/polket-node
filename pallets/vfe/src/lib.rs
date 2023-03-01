// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Business Process
//! 1. Sports brand create VFEBrand.
//! 2. Producer registration.
//! 3. Sports brand approved producer can mint VFE of it's brand.
//! 4. Producer registers the devices.
//! 5. The consumer buys the device and binds it.
//! 6. Consumers train daily, submit training reports, and get rewards..
//! 7. Consumers regain energy.
//! 8. Consumer charging VFE.
//! 9. Consumers upgrade VFE.
//! 10. Consumer Enhanced VFE Capability Value.
//! 11. Consumers unbind devices for VFE.

#![cfg_attr(not(feature = "std"), no_std)]

use bitcoin_hashes::ripemd160 as Ripemd;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect as MultiAssets, Mutate as MultiAssetsMutate, Transfer},
		tokens::nonfungibles::{
			Create, Inspect, InspectEnumerable, Mutate, Transfer as NFTTransfer,
		},
		Randomness, UnixTime,
	},
	transactional, PalletId,
};

use frame_system::pallet_prelude::*;

use bitcoin_hashes::Hash as OtherHash;
use frame_support::traits::fungibles;
pub use impl_nonfungibles::*;
use p256::ecdsa::{
	signature::{Signature as Sig, Verifier},
	Signature, VerifyingKey,
};
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use pallet_uniques::WeightInfo;
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedSub, Hash, One,
		Saturating, StaticLookup, Zero,
	},
	ModuleError, Permill, SaturatedConversion,
};
use sp_std::{
	borrow::ToOwned,
	boxed::Box,
	convert::{TryFrom, TryInto},
	ops::Mul,
	vec::Vec,
};
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod impl_nonfungibles;
pub mod types;

type BalanceOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::AssetId;
type VFEBrandApprovalOf<T> = VFEBrandApprove<AssetIdOf<T>, BalanceOf<T>>;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config<Self::UniquesInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		///  The origin which who can create collection of VFE.
		type BrandOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// The origin which may add or remove producer.
		type ProducerOrigin: EnsureOrigin<Self::Origin>;

		/// Multiple Asset hander, which should implement `frame_support::traits::fungibles`
		type Currencies: MultiAssets<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ MultiAssetsMutate<Self::AccountId>;

		/// Unify the value types of ProudcerId, CollectionId, ItemId, AssetId
		type ObjectId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ParentId = Self::Hash, ObjectId = Self::ObjectId>;

		/// The pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Used to randomly generate VFE base ability value
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// pallet-uniques instance
		type UniquesInstance: Copy + Clone + PartialEq + Eq;

		/// The producer-id parent key
		#[pallet::constant]
		type ProducerId: Get<Self::Hash>;

		/// The vfe brand-id parent key
		#[pallet::constant]
		type VFEBrandId: Get<Self::Hash>;

		/// Fees for unbinding VFE
		#[pallet::constant]
		type UnbindFee: Get<BalanceOf<Self>>;

		/// Units of Incentive Tokens Rewarded or Costed
		#[pallet::constant]
		type CostUnit: Get<BalanceOf<Self>>;

		/// How long to restore an energy value
		#[pallet::constant]
		type EnergyRecoveryDuration: Get<Self::BlockNumber>;

		/// How long to reset user daily earned value
		#[pallet::constant]
		type DailyEarnedResetDuration: Get<Self::BlockNumber>;

		/// level up cost factor
		#[pallet::constant]
		type LevelUpCostFactor: Get<BalanceOf<Self>>;

		/// init energy when new user created
		#[pallet::constant]
		type InitEnergy: Get<u16>;

		/// init earning cap of daily when new user created
		#[pallet::constant]
		type InitEarningCap: Get<u16>;

		/// ratio of each energy recovery
		#[pallet::constant]
		type EnergyRecoveryRatio: Get<Permill>;

		/// Used to get real world time
		type UnixTime: UnixTime;

		/// How long is the training report valid, unit: seconds
		#[pallet::constant]
		type ReportValidityPeriod: Get<u32>;

		/// Profit ratio of minting fee to VFE owner
		#[pallet::constant]
		type UserVFEMintedProfitRatio: Get<Permill>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_incentive_token)]
	/// Record the AssetId used to award incentive tokens to users.
	pub type IncentiveToken<T> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_nonce)]
	/// Self-incrementing nonce to obtain non-repeating random seeds
	pub type Nonce<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_last_energy_recovery)]
	/// Record the block number of the latest recoverable energy updated in the network
	pub type LastEnergyRecovery<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_last_daily_earned_reset)]
	/// Record the block number of the daily earning limit updated of the network
	pub type LastDailyEarnedReset<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_producers)]
	/// Records the currently registered producer.
	pub(crate) type Producers<T: Config> =
		StorageMap<_, Twox64Concat, T::ObjectId, Producer<T::ObjectId, T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_vfe_brands)]
	/// Record the currently created VFE brand.
	pub(crate) type VFEBrands<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		VFEBrand<T::CollectionId, T::StringLimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_users)]
	/// Record the user's daily training status
	pub(crate) type Users<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		User<T::AccountId, T::BlockNumber, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_devices)]
	/// Record device status
	pub(crate) type Devices<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		DeviceKey,
		Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_vfe_details)]
	/// Record the detailed attribute value of VFE item
	pub(super) type VFEDetails<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::CollectionId,
		Twox64Concat,
		T::ItemId,
		VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_vfe_approvals)]
	/// Record the allowed minting information of VFE brand
	pub(super) type VFEApprovals<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::CollectionId,
		Twox64Concat,
		T::ObjectId,
		VFEBrandApprovalOf<T>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// set incentive token.
		IncentiveTokenSet { asset_id: AssetIdOf<T> },

		/// Register Producer.
		ProducerRegister { who: T::AccountId, producer_id: T::ObjectId },

		/// producer change the owner.
		ProducerOwnerChanged {
			old_owner: T::AccountId,
			producer_id: T::ObjectId,
			new_owner: T::AccountId,
		},

		/// Created device type class.
		VFEBrandCreated {
			who: T::AccountId,
			brand_id: T::CollectionId,
			sport_type: SportType,
			rarity: VFERarity,
			note: Vec<u8>,
		},

		/// Register device.
		DeviceRegistered {
			operator: T::AccountId,
			producer_id: T::ObjectId,
			device_key: DeviceKey,
			brand_id: T::CollectionId,
		},

		/// deregister device.
		DeviceDeregistered { operator: T::AccountId, device_key: DeviceKey },

		/// Create VFE.
		VFECreated {
			owner: T::AccountId,
			detail: VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>,
		},

		/// Minted Art Toy vfe token.
		Issued { brand_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },

		/// An asset `instance` was transferred.
		Transferred {
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			from: T::AccountId,
			to: T::AccountId,
		},

		/// An asset `instance` was destroyed.
		Burned { brand_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },

		/// Bind the device with vfe.
		DeviceBound {
			owner: T::AccountId,
			device_key: DeviceKey,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
		},

		/// UnBind the device with vfe.
		DeviceUnbound {
			owner: T::AccountId,
			device_key: DeviceKey,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
		},

		/// Training reports and rewards with vfe.
		TrainingReportsAndRewards {
			owner: T::AccountId,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			sport_type: SportType,
			training_time: u32,
			training_duration: u16,
			training_count: u16,
			energy_used: u16,
			asset_id: AssetIdOf<T>,
			rewards: BalanceOf<T>,
		},

		/// PowerRecovery from device with vfe.
		PowerRestored {
			owner: T::AccountId,
			charge_amount: u16,
			use_amount: BalanceOf<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
		},

		/// user energy restored.
		UserEnergyRestored { who: T::AccountId, restored_amount: u16 },

		/// user daily earned reset.
		UserDailyEarnedReset { who: T::AccountId },

		/// Approved mint.
		ApprovedMint {
			brand_id: T::CollectionId,
			product_id: T::ObjectId,
			mint_amount: u32,
			mint_cost: Option<(AssetIdOf<T>, BalanceOf<T>)>,
		},

		/// Global energy recovery has occurred.
		GlobalEnergyRecoveryOccurred { block_number: T::BlockNumber },

		/// Global daily reset has occurred.
		GlobalDailyEarnedResetOccurred { block_number: T::BlockNumber },

		/// the VFE has been level up.
		VFELevelUp {
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			level_up: u16,
			cost: BalanceOf<T>,
		},

		/// the VFE ability is increased.
		VFEAbilityIncreased { brand_id: T::CollectionId, item_id: T::ItemId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// OperationIsNotAllowed
		OperationIsNotAllowed,
		/// VFEBrandNotFound
		VFEBrandNotFound,
		/// ValueInvalid
		ValueInvalid,
		/// RoleInvalid
		RoleInvalid,
		/// Error names should be descriptive.
		NoneValue,
		/// ValueOverflow
		ValueOverflow,
		/// ProducerNotExist
		ProducerNotExist,
		/// Device is not existed
		DeviceNotExisted,
		/// Device is existed
		DeviceExisted,
		/// the signature signed in device invalid
		DeviceSignatureInvalid,
		/// NonceMustGreatThanBefore
		NonceMustGreatThanBefore,
		/// item not found
		ItemNotFound,
		/// Device is not bond
		DeviceNotBond,
		/// VFE is not bond
		VFENotBond,
		/// VFE is bond
		VFEBond,
		/// VFENotExist
		VFENotExist,
		/// VFE is not fully charged
		VFENotFullyCharged,
		/// VFEUpgrading
		VFEUpgrading,
		/// VFE is fully charged
		VFEFullyCharged,
		/// UserNotExist
		UserNotExist,
		/// PublicKeyEncodeError
		PublicKeyEncodeError,
		/// Device has been bond
		DeviceBond,
		/// Device has been voided
		DeviceVoided,
		/// RemainingMintAmountIsNotZero
		RemainingMintAmountIsNotZero,
		/// user energy is full
		UserEnergyIsFull,
		/// incentive token not set
		IncentiveTokenNotSet,
		/// Energy is exhausted.
		EnergyExhausted,
		/// earned cap
		EarnedCap,
		/// Insufficient training
		InsufficientTraining,
		/// low battery
		LowBattery,
		/// training report time is expired
		TrainingReportTimeExpired,
		/// Training report out of normal range
		TrainingReportOutOfNormalRange,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut weight: Weight = 0;
			if (n % T::EnergyRecoveryDuration::get()).is_zero() {
				//update
				LastEnergyRecovery::<T>::put(n);
				Self::deposit_event(Event::GlobalEnergyRecoveryOccurred { block_number: n });
				weight = weight + T::DbWeight::get().writes(1);
			}

			if (n % T::DailyEarnedResetDuration::get()).is_zero() {
				//update
				LastDailyEarnedReset::<T>::put(n);
				Self::deposit_event(Event::GlobalDailyEarnedResetOccurred { block_number: n });
				weight = weight + T::DbWeight::get().writes(1);
			}

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::CollectionId: From<T::ObjectId>,
		T::ItemId: From<T::ObjectId>,
		T::ObjectId: From<T::CollectionId>,
	{
		/// set incentive token
		/// - origin AccountId sudo key can do
		#[pallet::weight(10_000)]
		pub fn set_incentive_token(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			IncentiveToken::<T>::put(asset_id);
			Self::deposit_event(Event::IncentiveTokenSet { asset_id });
			Ok(())
		}

		/// register_producer -Register the Producer
		/// - origin AccountId -creater
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn producer_register(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			// check if origin can register `who` to be a producer
			T::ProducerOrigin::ensure_origin(origin.clone())?;
			// auto increase ID
			let index = T::UniqueId::generate_object_id(T::ProducerId::get())?;

			Producers::<T>::insert(
				index.clone(),
				Producer { owner: who.clone(), id: index.clone() },
			);

			Self::deposit_event(Event::ProducerRegister { who, producer_id: index });
			Ok(())
		}

		/// register_producer -Register the Producer
		/// - origin AccountId -creater
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn producer_owner_change(
			origin: OriginFor<T>,
			id: T::ObjectId,
			new_owner: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			// Get identity role of origin
			let owner = ensure_signed(origin)?;
			let new_owner = T::Lookup::lookup(new_owner)?;
			let mut producer = Self::check_producer(owner.clone(), id.clone())?;
			// change the new owner
			producer.owner = new_owner.clone();
			Producers::<T>::insert(id, producer);

			// emit event
			Self::deposit_event(Event::ProducerOwnerChanged {
				old_owner: owner,
				producer_id: id,
				new_owner,
			});

			Ok(())
		}

		/// create a VFE brand
		/// - origin AccountId
		/// - class_id CollectionId
		/// - meta_data Vec<u8>
		#[pallet::weight(< T as pallet_uniques::Config < T::UniquesInstance >>::WeightInfo::create()
		+ < T as pallet_uniques::Config < T::UniquesInstance >>::WeightInfo::set_collection_metadata())]
		#[transactional]
		pub fn create_vfe_brand(
			origin: OriginFor<T>,
			meta_data: BoundedVec<u8, T::StringLimit>,
			sport_type: SportType,
			rarity: VFERarity,
		) -> DispatchResult {
			// Get identity role of origin
			let who = T::BrandOrigin::ensure_origin(origin.clone())?;
			let brand_id = T::UniqueId::generate_object_id(T::VFEBrandId::get())?;
			// let meta_data = meta_data.unwrap_or(Default::default());

			pallet_uniques::Pallet::<T, T::UniquesInstance>::create_collection(
				&brand_id.into(),
				&who,
				&who,
			)?;
			pallet_uniques::Pallet::<T, T::UniquesInstance>::set_collection_metadata(
				origin.clone(),
				brand_id.into(),
				meta_data.clone(),
				false,
			)?;
			let cid: T::CollectionId = brand_id.into();
			VFEBrands::<T>::insert(
				&cid,
				VFEBrand {
					brand_id: brand_id.into(),
					sport_type: sport_type.clone(),
					rarity: rarity.clone(),
					approvals: 0,
					uri: meta_data.clone(),
				},
			);

			Self::deposit_event(Event::VFEBrandCreated {
				who,
				brand_id: brand_id.into(),
				sport_type,
				rarity,
				note: Vec::<u8>::from(meta_data),
			});

			Ok(())
		}

		/// approve_mint
		/// - origin AccountId
		/// - class_id ClassId
		/// - delegate AccountId
		/// - mint_amount u32
		/// - mint_cost Option<(AssetId, Balance)>
		#[pallet::weight(10_000)]
		pub fn approve_mint(
			origin: OriginFor<T>,
			#[pallet::compact] brand_id: T::CollectionId,
			#[pallet::compact] producer_id: T::ObjectId,
			#[pallet::compact] mint_amount: u32,
			mint_cost: Option<(AssetIdOf<T>, BalanceOf<T>)>,
		) -> DispatchResult {
			let operator: T::AccountId = T::BrandOrigin::ensure_origin(origin)?;
			Self::do_approve_mint(brand_id, &operator, &producer_id, mint_amount, mint_cost)
		}

		/// register_device
		/// - origin AccountId
		/// - puk   DeviceKey
		/// - producer_id ProducerId
		/// - brand_id CollectionId
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn register_device(
			origin: OriginFor<T>,
			puk: DeviceKey,
			producer_id: T::ObjectId,
			brand_id: T::CollectionId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Devices::<T>::contains_key(puk), Error::<T>::DeviceExisted);
			let producer = Self::check_producer(who.clone(), producer_id)?;
			let vfe_brand =
				VFEBrands::<T>::get(brand_id.clone()).ok_or(Error::<T>::VFEBrandNotFound)?;

			// Check if the collection is authorized to the producer
			VFEApprovals::<T>::try_mutate(
				&vfe_brand.brand_id,
				&producer_id,
				|maybe_approved| -> DispatchResult {
					let mut approved = maybe_approved.take().ok_or(Error::<T>::NoneValue)?;

					// remaining_mint--
					let remaining = approved
						.remaining_mint
						.checked_sub(One::one())
						.ok_or(Error::<T>::ValueOverflow)?;

					// mint_cost handle transfer
					if let Some((mint_asset_id, mint_price)) = approved.mint_cost {
						// transfer tokens to NFT brand_id owner
						<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
							mint_asset_id,
							&who,
							&Self::into_account_id(producer_id),
							mint_price,
							true,
						)?;
						approved.locked_of_mint = approved
							.locked_of_mint
							.checked_add(&mint_price)
							.ok_or(Error::<T>::ValueOverflow)?;
					}

					approved.registered = approved
						.registered
						.checked_add(One::one())
						.ok_or(Error::<T>::ValueOverflow)?;
					approved.remaining_mint = remaining;

					Devices::<T>::insert(
						puk.clone(),
						Device {
							brand_id,
							item_id: None,
							producer_id: producer.id.clone(),
							status: DeviceStatus::Registered,
							pk: puk,
							nonce: 0u32,
							sport_type: vfe_brand.sport_type,
							timestamp: 0u32,
							mint_cost: approved.mint_cost,
						},
					);

					*maybe_approved = Some(approved);

					Self::deposit_event(Event::DeviceRegistered {
						operator: who,
						producer_id,
						device_key: puk,
						brand_id,
					});

					Ok(())
				},
			)
		}

		/// deregister_device
		/// - origin AccountId
		/// - puk   DeviceKey
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn deregister_device(origin: OriginFor<T>, puk: DeviceKey) -> DispatchResult {
			// deregister device only the producer of device
			let who = ensure_signed(origin.clone())?;
			Devices::<T>::try_mutate_exists(puk, |maybe_device| -> DispatchResult {
				let device = maybe_device.take().ok_or(Error::<T>::DeviceNotExisted)?;
				//check device status should Registered
				ensure!(device.status == DeviceStatus::Registered, Error::<T>::DeviceBond);
				//check device producer
				Self::check_producer(who.clone(), device.producer_id)?;
				// get approval
				let mut approved = VFEApprovals::<T>::get(&device.brand_id, &device.producer_id)
					.ok_or(Error::<T>::NoneValue)?;
				if let Some((mint_asset_id, mint_price)) = device.mint_cost {
					// transfer tokens to NFT class owner
					<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
						mint_asset_id,
						&Self::into_account_id(device.producer_id),
						&who,
						mint_price,
						false,
					)?;
					approved.locked_of_mint = approved
						.locked_of_mint
						.checked_sub(&mint_price)
						.ok_or(Error::<T>::ValueOverflow)?;
				}
				approved.registered =
					approved.registered.checked_sub(One::one()).ok_or(Error::<T>::ValueOverflow)?;
				approved.remaining_mint = approved
					.remaining_mint
					.checked_add(One::one())
					.ok_or(Error::<T>::ValueOverflow)?;
				VFEApprovals::<T>::insert(&device.brand_id, &device.producer_id, approved);
				//remove device from store
				*maybe_device = None;
				//emit event
				Self::deposit_event(Event::DeviceDeregistered { operator: who, device_key: puk });
				Ok(())
			})
		}

		/// bind_device
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn bind_device(
			origin: OriginFor<T>,
			from: T::AccountId,
			puk: DeviceKey,
			signature: BoundedVec<u8, T::StringLimit>,
			nonce: u32,
			bind_item: Option<T::ItemId>,
		) -> DispatchResult {
			ensure_none(origin)?;
			//  bind device signature
			let mut device = Self::get_verified_device(from.clone(), puk, signature, nonce)?;
			ensure!(device.item_id.is_none(), Error::<T>::DeviceBond);
			// create the user if it is new
			Self::find_user(&from);

			//If it is a registered device, it will mint a new vfe for the user.
			let new_vfe = if device.status == DeviceStatus::Registered {
				let vfe = Self::create_vfe(&device.brand_id, &device.producer_id, &from)?;
				// save new vfe detail
				VFEDetails::<T>::insert(&vfe.brand_id, &vfe.item_id, vfe.clone());
				Self::deposit_event(Event::VFECreated { owner: from.clone(), detail: vfe });
				Some(vfe)
			} else {
				None
			};

			// If the user passes itemId, bind this itemId, if not, bind a new vfe.
			let vfe = match bind_item {
				Some(item_id) => {
					//check if item_id is belong to origin
					let mut vfe = VFEDetails::<T>::get(&device.brand_id, &item_id)
						.ok_or(Error::<T>::VFENotExist)?;
					ensure!(vfe.device_key.is_none(), Error::<T>::VFEBond);
					let owner = Self::owner(&device.brand_id, &item_id)
						.ok_or(Error::<T>::OperationIsNotAllowed)?;
					ensure!(owner == from, Error::<T>::OperationIsNotAllowed);
					vfe.device_key = Some(puk);
					vfe
				},
				None => {
					//check if device status is register, we can get a new vfe to bind.
					let mut vfe = new_vfe.ok_or(Error::<T>::DeviceBond)?;
					vfe.device_key = Some(puk);

					vfe
				},
			};

			// save vfe detail after bond
			VFEDetails::<T>::insert(&vfe.brand_id, &vfe.item_id, vfe.clone());

			device.nonce = nonce;
			device.item_id = Some(vfe.item_id);
			device.status = DeviceStatus::Activated;
			// save device
			Devices::<T>::insert(puk, device);
			Self::deposit_event(Event::DeviceBound {
				owner: from,
				device_key: puk.clone(),
				brand_id: device.brand_id,
				item_id: vfe.item_id,
			});

			Ok(())
		}

		/// unbind the device
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn unbind_device(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let mut vfe =
				VFEDetails::<T>::get(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
			let device_pk = vfe.device_key.ok_or(Error::<T>::VFENotBond)?;
			let mut device = Devices::<T>::get(device_pk).ok_or(Error::<T>::DeviceNotExisted)?;
			// check vfe owner
			let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
			ensure!(vfe_owner == who, Error::<T>::OperationIsNotAllowed);
			//todo: pay fee to unbind device

			device.item_id = None;
			vfe.device_key = None;
			Devices::<T>::insert(device_pk, &device);
			VFEDetails::<T>::insert(&brand_id, &item_id, vfe);
			// VFEBindDevices::<T>::remove(&brand_id, &item_id);

			Self::deposit_event(Event::DeviceUnbound {
				owner: who,
				device_key: device_pk,
				brand_id: device.brand_id,
				item_id,
			});

			Ok(())
		}

		/// upload training report to the chain
		///  - origin AccountId
		/// - puk BoundedVec<u8, T::StringLimit>
		/// - req_sig BoundedVec<u8, T::StringLimit>
		/// - msg AccountId BoundedVec<u8, T::StringLimit>
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn upload_training_report(
			origin: OriginFor<T>,
			device_pk: DeviceKey,
			report_sig: BoundedVec<u8, T::StringLimit>,
			report_data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			ensure_none(origin.clone())?;
			let mut device =
				Self::check_device_training_report(device_pk, report_sig, report_data.clone())?;

			// decode the msg and earn the award
			Self::handler_report_data(&mut device, report_data)?;
			Ok(())
		}

		/// restore power
		/// - origin AccountId
		/// - brand_id CollectionId
		/// - item ItemId
		/// - charge_num u16
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn restore_power(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item: T::ItemId,
			#[pallet::compact] charge_num: u16,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let owner = Self::owner(&brand_id, &item).ok_or(Error::<T>::ItemNotFound)?;

			ensure!(who == owner, Error::<T>::OperationIsNotAllowed);

			let mut vfe = VFEDetails::<T>::get(brand_id, item).ok_or(Error::<T>::VFENotExist)?;

			ensure!(!vfe.is_upgrading, Error::<T>::VFEUpgrading);
			ensure!(vfe.remaining_battery < 100u16, Error::<T>::VFEFullyCharged);
			ensure!((vfe.remaining_battery + charge_num) <= 100u16, Error::<T>::ValueOverflow);

			let total_charge_cost = Self::calculate_charging_costs(vfe.clone(), charge_num);

			// try to burn the charge
			let incentive_token =
				IncentiveToken::<T>::get().ok_or(Error::<T>::IncentiveTokenNotSet)?;
			T::Currencies::burn_from(incentive_token, &owner, total_charge_cost)?;

			vfe.remaining_battery = vfe.remaining_battery + charge_num;

			// save common_prize
			VFEDetails::<T>::insert(brand_id.clone(), item.clone(), vfe);

			Self::deposit_event(Event::PowerRestored {
				owner,
				charge_amount: charge_num,
				use_amount: total_charge_cost,
				brand_id,
				item_id: item,
			});
			Ok(())
		}

		/// user restore energy and reset daily earned.
		/// - origin AccountId
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn user_restore(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			Self::_restore_energy(&who)?;
			Self::_reset_daily_earned(&who)?;
			Ok(())
		}

		/// level up
		/// - origin AccountId
		/// - brand_id CollectionId
		/// - instance ItemId
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn level_up(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
		) -> DispatchResult {
			// cost fee to level up vfe
			VFEDetails::<T>::try_mutate(&brand_id, &item_id, |maybe_vfe| -> DispatchResult {
				let who = ensure_signed(origin.clone())?;
				let mut vfe = maybe_vfe.take().ok_or(Error::<T>::VFENotExist)?;
				let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
				ensure!(vfe_owner == who, Error::<T>::OperationIsNotAllowed);
				let mut user = Self::find_user(&who);

				// Calculating level up fees for VFE
				let level_cost = Self::calculate_level_up_costs(&vfe, &user);

				// level up should burn token
				let incentive_token =
					IncentiveToken::<T>::get().ok_or(Error::<T>::IncentiveTokenNotSet)?;
				T::Currencies::burn_from(incentive_token, &who, level_cost)?;

				vfe.level = vfe.level + 1;
				vfe.available_points = vfe.available_points + 1 * vfe.rarity.growth_points();
				*maybe_vfe = Some(vfe);

				// increase the user's energy cap and earing cap of daily
				// check if user current energy_total is less than new energy_total
				let new_energy_cap = Self::level_into_energy_cap(vfe.level);
				let new_earning_cap = Self::level_into_earning_cap(vfe.level);
				if new_energy_cap > user.energy_total {
					user.energy_total = new_energy_cap;
				}
				if new_earning_cap > user.earning_cap {
					user.earning_cap = new_earning_cap;
				}
				Users::<T>::insert(&who, user);

				// emit event
				Self::deposit_event(Event::VFELevelUp {
					brand_id,
					item_id,
					level_up: vfe.level,
					cost: level_cost,
				});

				//todo: VFE level up requires a cooldown.

				Ok(())
			})
		}

		/// Increase ability
		/// - origin AccountId
		/// - brand_id CollectionId
		/// - instance ItemId
		/// - ability VFEAbility
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn increase_ability(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			ability: VFEAbility,
		) -> DispatchResult {
			VFEDetails::<T>::try_mutate(&brand_id, &item_id, |maybe_vfe| -> DispatchResult {
				let who = ensure_signed(origin.clone())?;
				let mut vfe = maybe_vfe.take().ok_or(Error::<T>::VFENotExist)?;
				let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
				ensure!(vfe_owner == who, Error::<T>::OperationIsNotAllowed);
				let total_ability =
					ability.efficiency + ability.skill + ability.luck + ability.durable;
				ensure!(total_ability <= vfe.available_points, Error::<T>::ValueInvalid);

				vfe.current_ability.efficiency =
					vfe.current_ability.efficiency.saturating_add(ability.efficiency);
				vfe.current_ability.skill = vfe.current_ability.skill.saturating_add(ability.skill);
				vfe.current_ability.luck = vfe.current_ability.luck.saturating_add(ability.luck);
				vfe.current_ability.durable =
					vfe.current_ability.durable.saturating_add(ability.durable);
				vfe.available_points = vfe.available_points.saturating_sub(total_ability);

				*maybe_vfe = Some(vfe);

				// emit event
				Self::deposit_event(Event::VFEAbilityIncreased { brand_id, item_id });

				Ok(())
			})
		}

		/// transfer vfe
		/// - origin AccountId
		/// - class CollectionId
		/// - instance ItemId
		/// - Source AccountId
		#[pallet::weight(< T as pallet_uniques::Config < T::UniquesInstance >>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			let to = T::Lookup::lookup(dest.clone())?;
			let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
			ensure!(vfe_owner == from, Error::<T>::OperationIsNotAllowed);
			<Self as NFTTransfer<T::AccountId>>::transfer(&brand_id, &item_id, &to)
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		T::CollectionId: From<T::ObjectId>,
		T::ItemId: From<T::ObjectId>,
		T::ObjectId: From<T::CollectionId>,
	{
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_device_tx = |provide| {
				ValidTransaction::with_tag_prefix("VFEDevice")
					.priority(UNSIGNED_TXS_PRIORITY)
					.and_provides([&provide])
					.longevity(TransactionLongevity::max_value())
					.propagate(true)
					.build()
			};

			match call.to_owned() {
				Call::bind_device { from, puk, signature, nonce, bind_item: _bind_item } => {
					Self::get_verified_device(from, puk, signature.clone(), nonce)
						.map_err(dispatch_error_to_invalid)?;
					valid_device_tx((puk, signature))
				},
				Call::upload_training_report { device_pk, report_sig, report_data } => {
					Self::check_device_training_report(
						device_pk.clone(),
						report_sig.clone(),
						report_data,
					)
					.map_err(dispatch_error_to_invalid)?;
					valid_device_tx((device_pk, report_sig))
				},

				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}

impl<T: Config> Pallet<T>
where
	T::CollectionId: From<T::ObjectId>,
	T::ItemId: From<T::ObjectId>,
	T::ObjectId: From<T::CollectionId>,
{
	fn max_generate_random() -> u32 {
		1000
	}

	pub fn do_mint(
		brand_id: T::CollectionId,
		item_id: T::ItemId,
		owner: T::AccountId,
	) -> DispatchResult {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::mint_into(&brand_id, &item_id, &owner)?;
		Self::deposit_event(Event::Issued { brand_id, item_id, owner });
		Ok(())
	}

	pub fn do_burn(brand_id: T::CollectionId, item_id: T::ItemId) -> DispatchResult {
		let owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::ItemNotFound)?;
		<pallet_uniques::Pallet<T, T::UniquesInstance> as Mutate<T::AccountId>>::burn(
			&brand_id, &item_id, None,
		)?;
		Self::deposit_event(Event::Burned { brand_id, item_id, owner });
		Ok(())
	}

	/// The account ID of the pallet.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// The account ID of the Producer.
	pub fn into_account_id(id: T::ObjectId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(id)
	}

	fn get_and_increment_nonce() -> Vec<u8> {
		let nonce = Nonce::<T>::get();
		Nonce::<T>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	fn generate_random_number() -> (u32, T::Hash, T::BlockNumber) {
		let nonce = Self::get_and_increment_nonce();
		let (random_seed, block_number) = T::Randomness::random(&nonce);
		let random_number = <u32>::decode(&mut random_seed.as_ref())
			.expect("secure hashes should always be bigger than u32; qed");
		(random_number, random_seed, block_number)
	}

	// Randomly value from among the total number.
	fn random_value(total: u16) -> u16 {
		let (mut random_number, _, _) = Self::generate_random_number();

		// Best effort attempt to remove bias from modulus operator.
		for _ in 1..Self::max_generate_random() {
			if random_number < u32::MAX - u32::MAX % (total as u32) {
				break
			}

			let (random_number2, _, _) = Self::generate_random_number();
			random_number = random_number2;
		}

		(random_number as u16) % total
	}

	fn verify_bind_device_message(
		account: T::AccountId,
		nonce: u32,
		puk: DeviceKey,
		signature: &[u8],
	) -> Result<bool, DispatchError> {
		let verify_key = VerifyingKey::from_sec1_bytes(puk.as_ref())
			.map_err(|_| Error::<T>::PublicKeyEncodeError)?;

		let sig =
			Signature::from_bytes(signature).map_err(|_| Error::<T>::DeviceSignatureInvalid)?;

		let account_nonce = nonce.to_le_bytes().to_vec();
		let account_rip160 = Ripemd::Hash::hash(account.encode().as_ref());

		let mut msg: Vec<u8> = Vec::new();
		msg.extend(account_nonce);
		msg.extend(account_rip160.to_vec());

		// check the validity of the signature
		let flag = verify_key.verify(&msg, &sig).is_ok();

		return Ok(flag)
	}

	// verifty the device binding signature and return device.
	fn get_verified_device(
		account: T::AccountId,
		puk: DeviceKey,
		signature: BoundedVec<u8, T::StringLimit>,
		nonce: u32,
	) -> Result<
		Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		DispatchError,
	> {
		// get the producer owner
		let device = Devices::<T>::get(puk.clone()).ok_or(Error::<T>::DeviceNotExisted)?;

		ensure!(device.status != DeviceStatus::Voided, Error::<T>::DeviceVoided);

		let flag = Self::verify_bind_device_message(account, nonce, puk, &signature[..])?;

		ensure!(flag, Error::<T>::DeviceSignatureInvalid);
		// check the nonce
		ensure!(nonce > device.nonce, Error::<T>::NonceMustGreatThanBefore);

		Ok(device)
	}

	// check the device's training report.
	fn check_device_training_report(
		puk: DeviceKey,
		req_sig: BoundedVec<u8, T::StringLimit>,
		msg: BoundedVec<u8, T::StringLimit>,
	) -> Result<
		Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		DispatchError,
	> {
		// get the producer owner
		let device = Devices::<T>::get(puk).ok_or(Error::<T>::DeviceNotExisted)?;

		ensure!(device.item_id.is_some(), Error::<T>::DeviceNotBond);

		let target = &req_sig[..];
		let sig = Signature::from_bytes(target).map_err(|_| Error::<T>::DeviceSignatureInvalid)?;

		let verify_key = VerifyingKey::from_sec1_bytes(puk.as_ref())
			.map_err(|_| Error::<T>::PublicKeyEncodeError)?;

		// check the validity of the signature
		let final_msg: &[u8] = &msg.as_ref();
		let flag = verify_key.verify(final_msg, &sig).is_ok();

		ensure!(flag, Error::<T>::DeviceSignatureInvalid);

		Ok(device)
	}

	// handler report data to get rewards
	fn handler_report_data(
		device: &mut Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		report_data: BoundedVec<u8, T::StringLimit>,
	) -> Result<(), DispatchError> {
		let brand_id = device.brand_id;
		let item_id = device.item_id.ok_or(Error::<T>::DeviceNotBond)?;
		let sport_type = device.sport_type;
		let account = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::ItemNotFound)?;

		// First try to restore user energy and daily earning cap.
		Self::_restore_energy(&account)?;
		Self::_reset_daily_earned(&account)?;

		match sport_type {
			SportType::JumpRope => {
				ensure!(report_data.len() == 17, Error::<T>::ValueInvalid);

				let training_report = JumpRopeTrainingReport::try_from(report_data.into_inner())
					.map_err(|_| Error::<T>::ValueInvalid)?;
				// Check whether data is submitted repeatedly
				ensure!(training_report.timestamp > device.timestamp, Error::<T>::ValueInvalid);
				let now = T::UnixTime::now().as_secs();
				let expired_time = training_report.timestamp + T::ReportValidityPeriod::get();
				ensure!(now >= training_report.timestamp as u64, Error::<T>::ValueInvalid);
				ensure!(now <= expired_time as u64, Error::<T>::TrainingReportTimeExpired);

				let mut vfe =
					VFEDetails::<T>::get(brand_id, item_id).ok_or(Error::<T>::VFENotExist)?;

				let mut user = Self::find_user(&account);

				ensure!(user.energy > 0, Error::<T>::EnergyExhausted);

				// check if earned cap
				ensure!(user.earned < user.earning_cap, Error::<T>::EarnedCap);

				// Power consumption = training-duration / training_unit_duration
				let mut power_used =
					training_report.jump_rope_duration / sport_type.training_unit_duration();

				// Check if the training is enough
				ensure!(power_used > 0, Error::<T>::InsufficientTraining);

				// check the user energy
				if power_used > user.energy {
					power_used = user.energy;
				}

				// check if VFE remaining battery is enough
				ensure!(vfe.remaining_battery > 0, Error::<T>::LowBattery);

				// check the vfe electric
				if power_used > vfe.remaining_battery {
					power_used = vfe.remaining_battery;
				}

				// update user energy and vfe remaining battery
				user.energy = user.energy - power_used;
				vfe.remaining_battery = vfe.remaining_battery.clone() - power_used;

				let r_luck = Self::random_value(vfe.current_ability.luck) + 1;
				let r_skill = (vfe.current_ability.skill * training_report.max_jump_rope_count) /
					((training_report.interruptions as u16 + 1) *
						sport_type.frequency_standard());
				let s = if vfe.current_ability.skill > r_skill {
					vfe.current_ability.skill -
						Self::random_value(vfe.current_ability.skill - r_skill)
				} else {
					vfe.current_ability.skill +
						Self::random_value(r_skill - vfe.current_ability.skill)
				};

				let f = sport_type.is_frequency_range(training_report.average_speed);
				ensure!(f > 0, Error::<T>::TrainingReportOutOfNormalRange);

				let e = vfe.current_ability.efficiency;

				let training_volume = (e + s + 2 * r_luck) * power_used * f;
				let cost_unit = T::CostUnit::get();
				let final_award = BalanceOf::<T>::from(training_volume).saturating_mul(cost_unit);

				//save user earned
				let earned = final_award.saturating_add(user.earned);
				let actual_award = if earned > user.earning_cap {
					user.earned = user.earning_cap;
					user.earning_cap.saturating_sub(user.earned)
				} else {
					user.earned = earned;
					final_award
				};

				// update the electric with user and vfe and device.
				device.timestamp = training_report.timestamp;
				Devices::<T>::insert(device.pk, device);
				Users::<T>::insert(account.clone(), user);
				VFEDetails::<T>::insert(brand_id.clone(), item_id.clone(), vfe);

				let reward_asset_id =
					IncentiveToken::<T>::get().ok_or(Error::<T>::IncentiveTokenNotSet)?;
				T::Currencies::mint_into(reward_asset_id, &account.clone(), actual_award.clone())?;

				Self::deposit_event(Event::TrainingReportsAndRewards {
					owner: account,
					brand_id,
					item_id,
					sport_type,
					training_time: training_report.timestamp,
					training_duration: training_report.jump_rope_duration,
					training_count: training_report.total_jump_rope_count,
					energy_used: power_used,
					asset_id: reward_asset_id,
					rewards: actual_award,
				});

				Ok(())
			},
			SportType::Running => Err(Error::<T>::ValueInvalid)?,
			SportType::Riding => Err(Error::<T>::ValueInvalid)?,
		}
	}

	// check the producer if it is exist and the owner meets the rules
	fn check_producer(
		owner: T::AccountId,
		id: T::ObjectId,
	) -> Result<Producer<T::ObjectId, T::AccountId>, DispatchError> {
		// get the producer owner
		let producer = Producers::<T>::get(id).ok_or(Error::<T>::ProducerNotExist)?;
		// check the machine owner
		ensure!(owner == producer.owner, Error::<T>::OperationIsNotAllowed);
		Ok(producer.into())
	}

	// find user by `account_id` if user not exist and create it
	fn find_user(account_id: &T::AccountId) -> User<T::AccountId, T::BlockNumber, BalanceOf<T>> {
		let maybe_user = Users::<T>::get(account_id);
		let user = maybe_user.unwrap_or_else(|| {
			let block_number = frame_system::Pallet::<T>::block_number();
			let user = User {
				owner: account_id.clone(),
				energy_total: Self::level_into_energy_cap(0),
				energy: Self::level_into_energy_cap(0),
				create_block: block_number,
				last_restore_block: T::BlockNumber::default(),
				last_earned_reset_block: T::BlockNumber::default(),
				earning_cap: Self::level_into_earning_cap(0),
				earned: Zero::zero(),
			};
			Users::<T>::insert(account_id, user.clone());
			user
		});
		user
	}

	// create VFE
	pub fn create_vfe(
		brand_id: &T::CollectionId,
		producer_id: &T::ObjectId,
		owner: &T::AccountId,
	) -> Result<VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>, DispatchError> {
		let vfe_brand = VFEBrands::<T>::get(brand_id).ok_or(Error::<T>::VFEBrandNotFound)?;
		let rarity = vfe_brand.rarity;

		let (min, max) = rarity.base_range_of_ability();
		let efficiency = min + Self::random_value(max - min);
		let skill = min + Self::random_value(max - min);
		let luck = min + Self::random_value(max - min);
		let durable = min + Self::random_value(max - min);
		let (_, gene, _) = Self::generate_random_number();

		// approve producer to mint new vfe
		let item_id = Self::do_mint_approved(brand_id.to_owned(), producer_id, &owner)?;

		let block_number = frame_system::Pallet::<T>::block_number();
		let base_ability = VFEAbility { efficiency, skill, luck, durable };
		let vfe = VFEDetail {
			brand_id: brand_id.to_owned(),
			item_id: item_id.clone(),
			base_ability: base_ability.clone(),
			current_ability: base_ability,
			rarity,
			level: 0,
			remaining_battery: 100,
			gene,
			last_block: block_number,
			is_upgrading: false,
			available_points: 0,
			device_key: None,
		};

		Ok(vfe)
	}

	pub fn do_approve_mint(
		brand_id: T::CollectionId,
		operator: &T::AccountId,
		producer_id: &T::ObjectId,
		mint_amount: u32,
		mint_cost: Option<(AssetIdOf<T>, BalanceOf<T>)>,
	) -> DispatchResult {
		//Check vfe brand owner
		let vfe_brand_owner =
			Self::collection_owner(&brand_id).ok_or(Error::<T>::VFEBrandNotFound)?;
		ensure!(operator == &vfe_brand_owner, Error::<T>::OperationIsNotAllowed);

		VFEApprovals::<T>::try_mutate(&brand_id, producer_id, |maybe_approved| -> DispatchResult {
			// find VFE brand
			let mut vfe_brand = VFEBrands::<T>::get(&brand_id).ok_or(Error::<T>::NoneValue)?;

			let mut approved = match maybe_approved.take() {
				// an approval already exists and is being updated
				Some(a) => a,
				// a new approval is created
				None => {
					vfe_brand.approvals.saturating_inc();
					VFEBrandApprove {
						mint_cost,
						remaining_mint: 0,
						locked_of_mint: BalanceOf::<T>::default(),
						activated: 0,
						registered: 0,
					}
				},
			};

			if mint_cost != None {
				// only total_can_mint == 0 can mutate total_can_mint
				ensure!(approved.remaining_mint == 0, Error::<T>::RemainingMintAmountIsNotZero);
				approved.mint_cost = mint_cost;
			}

			approved.remaining_mint = approved.remaining_mint.saturating_add(mint_amount);
			*maybe_approved = Some(approved);

			VFEBrands::<T>::insert(&brand_id, vfe_brand);
			Self::deposit_event(Event::ApprovedMint {
				brand_id,
				product_id: producer_id.to_owned(),
				mint_amount,
				mint_cost,
			});

			Ok(())
		})
	}

	// approve to mint a new instance
	fn do_mint_approved(
		vfe_brand_id: T::CollectionId,
		producer_id: &T::ObjectId,
		who: &T::AccountId,
	) -> Result<T::ItemId, DispatchError> {
		VFEApprovals::<T>::try_mutate(
			&vfe_brand_id,
			producer_id,
			|maybe_approved| -> Result<T::ItemId, DispatchError> {
				let mut approved = maybe_approved.take().ok_or(Error::<T>::NoneValue)?;
				let registered =
					approved.registered.checked_sub(One::one()).ok_or(Error::<T>::ValueOverflow)?;

				let activated =
					approved.activated.checked_add(One::one()).ok_or(Error::<T>::ValueOverflow)?;

				let vfe_brand_owner =
					Self::collection_owner(&vfe_brand_id).ok_or(Error::<T>::VFEBrandNotFound)?;
				let parent_id = Self::into_parent_id(T::VFEBrandId::get(), vfe_brand_id.into());
				let instance = T::UniqueId::generate_object_id(parent_id)?;
				Self::do_mint(vfe_brand_id.clone(), instance.into(), who.clone())?;

				// mint_cost handle transfer
				if let Some((mint_asset_id, mint_price)) = approved.mint_cost {
					let vfe_brand_owner_radio =
						Permill::from_percent(100) - T::UserVFEMintedProfitRatio::get();
					let vfe_brand_owner_profit = vfe_brand_owner_radio.mul(mint_price);
					// transfer tokens to VFE brand owner
					<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
						mint_asset_id,
						&Self::into_account_id(producer_id.to_owned()),
						&vfe_brand_owner,
						vfe_brand_owner_profit,
						false,
					)?;

					// transfer tokens to VFE item of owner
					let vfe_item_owner_profit = T::UserVFEMintedProfitRatio::get().mul(mint_price);
					<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
						mint_asset_id,
						&Self::into_account_id(producer_id.to_owned()),
						who,
						vfe_item_owner_profit,
						false,
					)?;

					let locked_of_mint = approved
						.locked_of_mint
						.checked_sub(&mint_price)
						.ok_or(Error::<T>::ValueOverflow)?;

					approved.locked_of_mint = locked_of_mint;
				}

				approved.registered = registered;
				approved.activated = activated;
				*maybe_approved = Some(approved);
				Ok(instance.into())
			},
		)
	}

	// restore user energy
	fn _restore_energy(who: &T::AccountId) -> DispatchResult {
		let mut user = Self::find_user(&who);
		let last_energy_recovery = LastEnergyRecovery::<T>::get();
		if user.energy < user.energy_total {
			let user_last_restore_block = user.last_restore_block;
			let duration = T::EnergyRecoveryDuration::get();

			let recoverable_times = last_energy_recovery.saturating_sub(user_last_restore_block);
			let recoverable_times =
				recoverable_times.checked_div(&duration).ok_or(Error::<T>::ValueOverflow)?;
			let average_recovery: u16 =
				T::EnergyRecoveryRatio::get().mul(user.energy_total as u32).saturated_into();
			let recoverable_energy = recoverable_times.saturated_into::<u16>() * average_recovery;
			let max_recovery_energy = recoverable_energy + user.energy;
			let restored_amount = if max_recovery_energy > user.energy_total {
				let restored_amount = user.energy_total - user.energy;
				user.energy = user.energy_total;
				restored_amount
			} else {
				user.energy = max_recovery_energy;
				recoverable_energy
			};

			Self::deposit_event(Event::UserEnergyRestored { who: who.to_owned(), restored_amount });
		}

		user.last_restore_block = last_energy_recovery;
		Users::<T>::insert(&who, user);

		Ok(())
	}

	// reset user daily earned
	fn _reset_daily_earned(who: &T::AccountId) -> DispatchResult {
		let mut user = Self::find_user(&who);

		let user_last_earned_reset_block = user.last_earned_reset_block;
		let last_daily_earned_reset = LastDailyEarnedReset::<T>::get();
		if user_last_earned_reset_block < last_daily_earned_reset {
			user.earned = Zero::zero();

			Self::deposit_event(Event::UserDailyEarnedReset { who: who.to_owned() });
		}

		user.last_earned_reset_block = last_daily_earned_reset;
		Users::<T>::insert(&who, user);
		Ok(())
	}

	pub fn get_vfe_details_by_address(
		account: T::AccountId,
		brand_id: T::CollectionId,
	) -> Vec<VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>> {
		let items = Self::owned_in_collection(&brand_id, &account);
		let mut values = Vec::new();
		items.for_each(|e| {
			if let Some(vfe) = VFEDetails::<T>::get(&brand_id, &e) {
				values.push(vfe);
			};
		});
		values
	}

	// level into energy cap of daily
	pub fn level_into_energy_cap(level: u16) -> u16 {
		// increase energy per 2 level
		(level / 2) * T::InitEnergy::get() / 2 + T::InitEnergy::get()
	}

	// level into earning cap of daily
	pub fn level_into_earning_cap(level: u16) -> BalanceOf<T> {
		let base_cap = T::InitEarningCap::get();
		let cap = base_cap * level + base_cap;
		BalanceOf::<T>::saturated_from(cap).saturating_mul(T::CostUnit::get())
	}

	// calculate VFE charging costs
	pub(crate) fn calculate_charging_costs(
		vfe: VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>,
		charge_num: u16,
	) -> BalanceOf<T> {
		let mut charge_num = charge_num;
		if vfe.remaining_battery + charge_num > 100u16 {
			charge_num = 100u16 - vfe.remaining_battery;
		}

		let p_one = (vfe.base_ability.efficiency +
			vfe.base_ability.skill +
			vfe.base_ability.luck +
			vfe.base_ability.durable) /
			2;

		let p_two = (vfe.current_ability.efficiency +
			vfe.current_ability.skill +
			vfe.current_ability.luck +
			vfe.current_ability.durable) /
			(4 * vfe.current_ability.durable);

		let p_two = p_two.pow(2) * vfe.level;
		let total_charge_cost =
			BalanceOf::<T>::from((p_one + p_two) * charge_num).saturating_mul(T::CostUnit::get());

		total_charge_cost
	}

	// calculate VFE level up costs
	pub(crate) fn calculate_level_up_costs(
		vfe: &VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>,
		user: &User<T::AccountId, T::BlockNumber, BalanceOf<T>>,
	) -> BalanceOf<T> {
		// Calculating level up fees for VFE
		let t = T::LevelUpCostFactor::get();
		let cost_unit = T::CostUnit::get();
		let base_ability = (vfe
			.base_ability
			.efficiency
			.saturating_add(vfe.base_ability.skill)
			.saturating_add(vfe.base_ability.luck)
			.saturating_sub(vfe.base_ability.durable)) /
			2;
		let g = vfe.rarity.growth_points();
		let n = user.energy_total;
		let level_up_cost = base_ability + vfe.level * (g - 1) * n;
		let level_cost =
			BalanceOf::<T>::from(level_up_cost).saturating_mul(t).saturating_mul(cost_unit);

		level_cost
	}

	// get VFE charging costs
	pub fn get_charging_costs(
		brand_id: T::CollectionId,
		item: T::ItemId,
		charge_num: u16,
	) -> BalanceOf<T> {
		let vfe = VFEDetails::<T>::get(brand_id, item);
		if vfe.is_none() {
			Zero::zero()
		} else {
			let vfe = vfe.unwrap();
			Self::calculate_charging_costs(vfe, charge_num)
		}
	}

	// get VFE level up costs
	pub fn get_level_up_costs(
		who: T::AccountId,
		brand_id: T::CollectionId,
		item: T::ItemId,
	) -> BalanceOf<T> {
		let vfe = VFEDetails::<T>::get(brand_id, item);
		let user = Self::find_user(&who);
		let vfe_owner = Self::owner(&brand_id, &item);
		if vfe.is_some() && vfe_owner.is_some() {
			let vfe_owner = vfe_owner.unwrap();
			let vfe = vfe.unwrap();
			if vfe_owner == who {
				Self::calculate_level_up_costs(&vfe, &user)
			} else {
				Zero::zero()
			}
		} else {
			Zero::zero()
		}
	}

	pub fn check_vfe_can_transfer(brand_id: &T::CollectionId, item: &T::ItemId) -> DispatchResult {
		// Only VFE is fully charged or not uprading or unbound can be transferred
		let vfe = VFEDetails::<T>::get(brand_id, item).ok_or(Error::<T>::VFENotExist)?;
		ensure!(vfe.remaining_battery >= 100, Error::<T>::VFENotFullyCharged);
		ensure!(!vfe.is_upgrading, Error::<T>::VFEUpgrading);
		ensure!(vfe.device_key.is_none(), Error::<T>::VFEBond);
		Ok(())
	}

	/// The parent ID of the VFE Brand Id.
	pub fn into_parent_id(
		parent_id: T::Hash,
		child_id: T::ObjectId,
	) -> <T as frame_system::Config>::Hash {
		let encoded = (parent_id, child_id).encode();
		let key: <T as frame_system::Config>::Hash =
			<T as frame_system::Config>::Hashing::hash(encoded.as_ref());
		key
	}
}

/// convert a DispatchError to a custom InvalidTransaction with the inner code being the error
/// number.
pub fn dispatch_error_to_invalid(error: DispatchError) -> InvalidTransaction {
	let error_number = match error {
		DispatchError::Module(ModuleError { error, .. }) => error[0],
		_ => 0,
	};
	InvalidTransaction::Custom(error_number)
}
