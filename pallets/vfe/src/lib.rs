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
use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect as MultiAssets, Mutate as MultiAssetsMutate, Transfer},
		tokens::nonfungibles::{Create, Inspect, Mutate},
		Randomness, ReservableCurrency,
	},
	transactional, PalletId, RuntimeDebug,
};

use frame_system::{pallet_prelude::*, RawOrigin};
use num_integer::Roots;

use bitcoin_hashes::{sha256 as Sha256, Hash as OtherHash};
use frame_support::traits::fungibles;
use p256::{
	ecdsa::{
		signature::{Signature as Sig, Signer, Verifier},
		Signature, SigningKey, VerifyingKey,
	},
	elliptic_curve::{sec1::ToEncodedPoint, PublicKey},
	NistP256,
};
pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use pallet_uniques::WeightInfo;
use scale_info::{prelude::format, TypeInfo};
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul,
						 CheckedSub, One, Saturating, StaticLookup, Verify};
use sp_std::{
	borrow::ToOwned,
	boxed::Box,
	convert::{TryFrom, TryInto},
	vec::Vec,
};
use sp_runtime::traits::Zero;
use sp_runtime::SaturatedConversion;
use sp_runtime::traits::CheckedDiv;
use types::*;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod impl_nonfungibles;
mod types;

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

		///  Who can create VFE class and register producer
		type BrandOrigin: EnsureOrigin<Self::Origin, Success=Self::AccountId>;

		/// Who can register device and mint VFEs
		type ProducerOrigin: EnsureOrigin<Self::Origin, Success=Self::AccountId>;

		/// Multiple asset types
		type Currencies: MultiAssets<Self::AccountId>
		+ Transfer<Self::AccountId>
		+ MultiAssetsMutate<Self::AccountId>;

		/// ObjectId linked Data
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ObjectId=Self::ObjectId>;

		/// The pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Randomness
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// pallet-uniques instance
		type UniquesInstance: Copy + Clone + PartialEq + Eq;

		/// The producer id
		#[pallet::constant]
		type ProducerId: Get<Self::ObjectId>;

		/// The vfe brand id
		#[pallet::constant]
		type VFEBrandId: Get<Self::ObjectId>;

		#[pallet::constant]
		type MaxGenerateRandom: Get<u32>;

		/// Fees for unbinding VFE
		#[pallet::constant]
		type UnbindFee: Get<BalanceOf<Self>>;

		/// cost base unit
		#[pallet::constant]
		type CostUnit: Get<BalanceOf<Self>>;

		/// How long to restore an energy value
		#[pallet::constant]
		type EnergyRecoveryDuration: Get<Self::BlockNumber>;

		/// level up cost factor
		#[pallet::constant]
		type LevelUpCostFactor: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn incentive_token)]
	pub type IncentiveToken<T> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub type Nonce<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_energy_recovery)]
	pub type LastEnergyRecovery<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn producers)]
	pub(crate) type Producers<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::ObjectId,
		Producer<T::ObjectId, T::AccountId>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn vfe_brands)]
	pub(crate) type VFEBrands<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		VFEBrand<T::CollectionId, T::StringLimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub(crate) type Users<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		User<T::AccountId, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn devices)]
	pub(crate) type Devices<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 33],
		Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn device_vfes)]
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
	#[pallet::getter(fn vfe_bind_devices)]
	pub(crate) type VFEBindDevices<T: Config> =
	StorageDoubleMap<
		_,
		Twox64Concat, T::CollectionId,
		Twox64Concat, T::ItemId,
		[u8; 33], OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_vfe_approvals)]
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
		/// Register Producer . \[creater, producer_id\]
		ProducerRegister(T::AccountId, T::ObjectId),

		/// producer change the owner  \[former_owner, producer_id,  new_owner\]
		ProducerOwnerChanged(T::AccountId, T::ObjectId, T::AccountId),

		///  producer charge the locked_of_mint   \[former_owner, producer_id,  asset_id, balance \]
		ProducerCharge(T::AccountId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>),

		/// producer withdraw the locked_of_mint  \[former_owner, producer_id,  asset_id, balance \]
		ProducerWithdraw(T::AccountId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>),

		/// Created device type class. \[executor, class_id, sport_type, note\]
		VFEBrandCreated(T::AccountId, T::CollectionId, SportType, VFERarity, Vec<u8>),

		/// Register device. \[operator, producer_id, public_key,  class\]
		DeviceRegistered(T::AccountId, T::ObjectId, [u8; 33], T::CollectionId),

		/// deregister device. \[operator, public_key\]
		DeviceDeregistered(T::AccountId, [u8; 33]),

		/// Create VFE. \[owner, VFE_detail\]
		VFECreated(T::AccountId, VFEDetail<T::CollectionId, T::ItemId, T::Hash, T::BlockNumber>),

		/// Minted Art Toy vfe token. \[class, instance, owner\]
		Issued(T::CollectionId, T::ItemId, T::AccountId),

		/// An asset `instance` was transferred. \[ class, instance, from, to \]
		Transferred(T::CollectionId, T::ItemId, T::AccountId, T::AccountId),

		/// An asset `instance` was destroyed. \[ class, instance, owner \]
		Burned(T::CollectionId, T::ItemId, T::AccountId),

		/// Bind the device with vfe. \[ owner,public_key, class, instance  \]
		DeviceBound(T::AccountId, [u8; 33], T::CollectionId, T::ItemId),

		/// UnBind the device with vfe. \[ owner,public_key, class,former instance  \]
		DeviceUnbound(T::AccountId, [u8; 33], T::CollectionId, T::ItemId),

		/// Training reports and rewards with vfe. \[ owner, brand_id, item_id, sport_type, training_time, training_duration, training_count, energy_used, asset_id, rewards \]
		TrainingReportsAndRewards(T::AccountId, T::CollectionId, T::ItemId, SportType, u32, u16, u16, u16, AssetIdOf<T>, BalanceOf<T>),

		/// PowerRecovery from device with vfe. \[ owner, use_amount,class, instance  \]
		PowerRestored(T::AccountId, u16, BalanceOf<T>, T::CollectionId, T::ItemId),

		/// user energy restored. \[ owner, restored_amount \]
		UserEnergyRestored(T::AccountId, u16),

		Sha256Test(Vec<u8>),

		/// VerifyTest. \[ pubkey, msg, sha256msg, signature, isValid  \]
		VerifyTest(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, bool),

		/// SignTest. \[ privatekey, pubkey, msg, sha256msg, signature \]
		SignTest(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>),

		/// ApprovedMint \[collection_id, product_id, mint_amount, mint_cost\]
		ApprovedMint(T::CollectionId, T::ObjectId, u32, Option<(AssetIdOf<T>, BalanceOf<T>)>),

		/// Global energy recovery has occurred \[block_number\]
		GlobalEnergyRecoveryOccurred(T::BlockNumber),

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
		/// DeviceNotExist
		DeviceNotExist,
		/// DeviceTimeStampMustGreaterThanBefore
		DeviceTimeStampMustGreaterThanBefore,
		/// OperationIsNotAllowedForProducer
		OperationIsNotAllowedForProducer,
		/// OperationIsNotAllowedForSign
		OperationIsNotAllowedForSign,
		/// NonceMustGreatThanBefore
		NonceMustGreatThanBefore,
		/// BalanceNotEnough
		BalanceNotEnough,
		/// PublicKeyExist
		PublicKeyExist,
		/// ToolSeriesNotExist
		ToolSeriesNotExist,
		/// ToolParamNotExist
		ToolParamNotExist,
		/// ToolParamValueNotExist
		ToolParamValueNotExist,
		/// OperationIsNotAllowedForTool
		OperationIsNotAllowedForTool,
		/// InstanceNotFound
		InstanceNotFound,
		/// ItemIdCannotBeNull
		ItemIdCannotBeNull,
		/// InstanceNotBelongAnyone
		InstanceNotBelongAnyone,
		/// InstanceNotBelongTheTarget
		InstanceNotBelongTheTarget,
		/// DeviceNotBond
		DeviceNotBond,
		/// DeviceMsgNotCanNotBeDecode
		DeviceMsgNotCanNotBeDecode,
		/// DeviceMsgDecodeErr
		DeviceMsgDecodeErr,
		/// VFENotExist
		VFENotExist,
		/// VFENotFullElectric
		VFENotFullElectric,
		/// VFEUpgrading
		VFEUpgrading,
		/// VFEUpdating
		VFEFullElectric,
		/// UserNotExist
		UserNotExist,
		/// PublicKeyEncodeError
		PublicKeyEncodeError,
		/// SigEncodeError
		SigEncodeError,
		/// CurrenciesNotSupport
		CurrenciesNotSupport,
		/// DeviceHasBeenBond
		DeviceHasBeenBond,
		/// RemainingMintAmountIsNotZero
		RemainingMintAmountIsNotZero,
		/// user energy is full
		UserEnergyIsFull,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			// Check to see if we should spend some funds!
			if (n % T::EnergyRecoveryDuration::get()).is_zero() {
				//update
				LastEnergyRecovery::<T>::put(n);
				Self::deposit_event(Event::GlobalEnergyRecoveryOccurred(n));
				T::DbWeight::get().writes(1)
			} else {
				0
			}
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
		/// -origin AccountId sudo key can do
		#[pallet::weight(10_000)]
		pub fn set_incentive_token(
			origin: OriginFor<T>,
			asset_id: T::ObjectId,
		) -> DispatchResult {
			ensure_root(origin)?;

			Ok(())
		}


		/// register_producer -Register the Producer
		/// - origin AccountId -creater
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn producer_register(origin: OriginFor<T>) -> DispatchResult {
			// Get identity role of origin
			let who = T::ProducerOrigin::ensure_origin(origin.clone())?;
			let index = T::UniqueId::generate_object_id(T::ProducerId::get())?;
			// let account_id = Self::into_account_id(index.clone());

			Producers::<T>::insert(
				index.clone(),
				Producer {
					owner: who.clone(),
					id: index.clone(),
				},
			);

			Self::deposit_event(Event::ProducerRegister(who, index));
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
			let owner = T::ProducerOrigin::ensure_origin(origin.clone())?;

			let new_owner = T::Lookup::lookup(new_owner)?;

			// check the role if it meets the rules
			T::ProducerOrigin::ensure_origin(RawOrigin::Signed(new_owner.clone()).into())
				.map_err(|_| Error::<T>::RoleInvalid)?;

			let mut producer = Self::check_producer(owner.clone(), id.clone())?;

			// change the new owner
			producer.owner = new_owner.clone();

			Producers::<T>::insert(id, producer);

			// save it to event
			Self::deposit_event(Event::ProducerOwnerChanged(owner, id, new_owner));

			Ok(())
		}

		// /// producer_charge - Owner charge the locked_of_mint to  the Producer
		// /// - origin AccountId -creater
		// /// - producer_id ProducerId -producer
		// /// - amount Balance -target amount
		// /// - asset_id AssetId - Asset ID
		// #[pallet::weight(10_000)]
		// #[transactional]
		// pub fn producer_charge(
		// 	origin: OriginFor<T>,
		// 	producer_id: ObjectId,
		// 	asset_id: AssetIdOf<T>,
		// 	amount: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	// Get identity role of origin
		// 	let owner = T::ProducerOrigin::ensure_origin(origin.clone())?;
		//
		// 	// get the producer
		// 	let mut producer = Self::check_producer(owner.clone(), producer_id)?;
		//
		// 	// check out the owner_balance
		// 	let owner_balance = T::Currencies::balance(asset_id, &owner);
		//
		// 	// check the owner's balance greater or equal to the target amount
		// 	ensure!(owner_balance >= amount, Error::<T>::BalanceNotEnough);
		//
		// 	let producer_account = producer.account.clone();
		//
		// 	// try to transfer the charge
		// 	T::Currencies::transfer(asset_id, &owner, &producer_account, amount, true)?;
		//
		// 	let producer_balance = T::Currencies::balance(asset_id, &producer_account);
		//
		// 	// change the locked_of_mint
		// 	producer.locked_of_mint = producer_balance;
		//
		// 	// update the producer
		// 	Producers::<T>::insert(producer.id, producer);
		//
		// 	// save it to event
		// 	Self::deposit_event(Event::ProducerCharge(owner, producer_id, asset_id, amount));
		//
		// 	Ok(())
		// }

		// /// producer_withdraw - Owner withdraw the locked_of_mint from the Producer
		// /// - origin AccountId -creater
		// /// - producer_id ProducerId -producer
		// /// - amount Balance -target amount
		// /// - asset_id AssetId - Asset ID
		// #[pallet::weight(10_000)]
		// #[transactional]
		// pub fn producer_withdraw(
		// 	origin: OriginFor<T>,
		// 	producer_id: ObjectId,
		// 	asset_id: AssetIdOf<T>,
		// 	amount: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	// Get identity role of origin
		// 	let owner = T::ProducerOrigin::ensure_origin(origin.clone())?;
		//
		// 	// get the producer
		// 	let mut producer = Self::check_producer(owner.clone(), producer_id)?;
		//
		// 	// check out the producer account
		// 	let producer_account = producer.account.clone();
		//
		// 	// get the producer locked_of_mint balance
		// 	let producer_balance = T::Currencies::balance(asset_id, &producer_account);
		//
		// 	// check the owner's balance greater or equal to the target amount
		// 	ensure!(producer_balance >= amount, Error::<T>::BalanceNotEnough);
		//
		// 	// try to transfer the charge
		// 	T::Currencies::transfer(asset_id, &producer_account, &owner, amount, true)?;
		//
		// 	// change the locked_of_mint
		// 	producer.locked_of_mint = producer_balance;
		//
		// 	// update the producer
		// 	Producers::<T>::insert(producer.id, producer);
		//
		// 	// save it to event
		// 	Self::deposit_event(Event::ProducerWithdraw(owner, producer_id, asset_id, amount));
		//
		// 	Ok(())
		// }


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

			pallet_uniques::Pallet::<T, T::UniquesInstance>::create_collection(&brand_id.into(), &who, &who)?;
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

			Self::deposit_event(Event::VFEBrandCreated(
				who,
				brand_id.into(),
				sport_type,
				rarity,
				Vec::<u8>::from(meta_data),
			));

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
			let operator = T::BrandOrigin::ensure_origin(origin)?;
			Self::do_approve_mint(brand_id, &operator, &producer_id, mint_amount, mint_cost)
		}

		/// register_device
		/// - origin AccountId
		/// - puk   BoundedVec<u8, T::StringLimit>
		/// - producer_id ProducerId
		/// - class CollectionId
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn register_device(
			origin: OriginFor<T>,
			puk: [u8; 33],
			producer_id: T::ObjectId,
			class: T::CollectionId,
		) -> DispatchResult {
			let who = T::ProducerOrigin::ensure_origin(origin.clone())?;
			ensure!(!Devices::<T>::contains_key(puk), Error::<T>::PublicKeyExist);
			let producer = Self::check_producer(who.clone(), producer_id)?;
			let vfe_brand =
				VFEBrands::<T>::get(class.clone()).ok_or(Error::<T>::VFEBrandNotFound)?;

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
						// transfer tokens to NFT class owner
						<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
							mint_asset_id,
							&who,
							&Self::into_account_id(producer_id),
							mint_price,
							true,
						)?;
						approved.locked_of_mint = approved.locked_of_mint.checked_add(&mint_price).ok_or(Error::<T>::ValueOverflow)?;
					}

					approved.registered = approved.registered.checked_add(One::one()).ok_or(Error::<T>::ValueOverflow)?;
					approved.remaining_mint = remaining;

					Devices::<T>::insert(
						puk.clone(),
						Device {
							brand_id: class,
							item_id: None,
							producer_id: producer.id.clone(),
							status: DeviceStatus::Registered,
							pk: puk,
							nonce: 0u32,
							sport_type: vfe_brand.sport_type,
							timestamp: 0u32,
							mint_cost: approved.mint_cost,
						});

					*maybe_approved = Some(approved);

					Self::deposit_event(Event::DeviceRegistered(who, producer_id, puk, class));

					Ok(())
				},
			)
		}

		/// deregister_device
		/// - origin AccountId
		/// - puk   BoundedVec<u8, T::StringLimit>
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn deregister_device(
			origin: OriginFor<T>,
			puk: [u8; 33],
		) -> DispatchResult {
			// deregister device only the producer of device
			let who = T::ProducerOrigin::ensure_origin(origin.clone())?;

			Devices::<T>::try_mutate_exists(puk, |maybe_device| -> DispatchResult {
				let device = maybe_device.take().ok_or(Error::<T>::DeviceNotExist)?;
				//check device status should Registered
				ensure!(device.status == DeviceStatus::Registered, Error::<T>::DeviceHasBeenBond);
				//check device producer
				Self::check_producer(who.clone(), device.producer_id)?;
				// get approval
				let mut approved = VFEApprovals::<T>::get(&device.brand_id, &device.producer_id).ok_or(Error::<T>::NoneValue)?;
				if let Some((mint_asset_id, mint_price)) = device.mint_cost {
					// transfer tokens to NFT class owner
					<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
						mint_asset_id,
						&Self::into_account_id(device.producer_id),
						&who,
						mint_price,
						false,
					)?;
					approved.locked_of_mint = approved.locked_of_mint.checked_sub(&mint_price).ok_or(Error::<T>::ValueOverflow)?;
				}
				approved.registered = approved.registered.checked_sub(One::one()).ok_or(Error::<T>::ValueOverflow)?;
				approved.remaining_mint = approved.remaining_mint.checked_add(One::one()).ok_or(Error::<T>::ValueOverflow)?;
				VFEApprovals::<T>::insert(&device.brand_id, &device.producer_id, approved);
				//remove device from store
				*maybe_device = None;
				//emit event
				Self::deposit_event(Event::DeviceDeregistered(who, puk));
				Ok(())
			})
		}

		/// bind_device
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn bind_device(
			origin: OriginFor<T>,
			puk: [u8; 33],
			signature: BoundedVec<u8, T::StringLimit>,
			nonce: u32,
			bind_item: Option<T::ItemId>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			//  bind device signature
			let mut device = Self::check_device_pub(from.clone(), puk, signature, nonce)?;

			// create the user if it is new
			Self::create_new_user(from.clone());

			// In this case, if the device is
			if device.status == DeviceStatus::Registered {
				// create the new instance
				let item =
					Self::create_vfe(&device.brand_id, &device.producer_id, &from)?;

				device.item_id = Some(item);
				device.status = DeviceStatus::Activated;
				Devices::<T>::insert(puk, device);

				//vfe bind device pubkey
				VFEBindDevices::<T>::insert(&device.brand_id, &item, puk);

				Self::deposit_event(Event::DeviceBound(
					from,
					puk.clone(),
					device.brand_id,
					item,
				));
			} else {
				ensure!(device.item_id.is_none(), Error::<T>::DeviceHasBeenBond);

				let instance = bind_item.ok_or(Error::<T>::NoneValue)?;

				device.item_id = Some(instance);
				Devices::<T>::insert(puk.clone(), device);
				Self::deposit_event(Event::DeviceBound(from, puk, device.brand_id, instance));
			}

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
			let device_pk = VFEBindDevices::<T>::get(&brand_id, &item_id).ok_or(Error::<T>::DeviceNotBond)?;
			let mut device = Devices::<T>::get(device_pk).ok_or(Error::<T>::DeviceNotExist)?;
			// check vfe owner
			let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
			ensure!(vfe_owner == who, Error::<T>::OperationIsNotAllowed);
			//todo: pay fee to unbind device

			device.item_id = None;
			Devices::<T>::insert(device_pk, &device);
			VFEBindDevices::<T>::remove(&brand_id, &item_id);

			Self::deposit_event(Event::DeviceUnbound(who, device_pk, device.brand_id, item_id));

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
			device_pk: [u8; 33],
			report_sig: BoundedVec<u8, T::StringLimit>,
			report_data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;

			let mut device = Self::check_device_data(from.clone(), device_pk, report_sig, report_data.clone())?;

			// decode the msg and earn the award
			Self::handler_report_data(&mut device, from, report_data)?;
			Ok(())
		}

		/// restore power
		/// - origin AccountId
		/// - class CollectionId
		/// - instance ItemId
		/// - Source AccountId
		#[pallet::weight(10_000)]
		pub fn restore_power(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			instance: T::ItemId,
			#[pallet::compact] charge_num: u16,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let owner = Self::owner(&brand_id, &instance).ok_or(Error::<T>::InstanceNotFound)?;

			ensure!(who == owner, Error::<T>::OperationIsNotAllowed);

			let mut vfe = VFEDetails::<T>::get(brand_id, instance).ok_or(Error::<T>::VFENotExist)?;

			ensure!(!vfe.is_upgrading, Error::<T>::VFEUpgrading);
			ensure!(vfe.remaining_battery < 100u16, Error::<T>::VFEFullElectric);
			ensure!((vfe.remaining_battery + charge_num) <= 100u16, Error::<T>::ValueOverflow);

			let p_one = (vfe.base_ability.efficiency
				+ vfe.base_ability.skill
				+ vfe.base_ability.luck
				+ vfe.base_ability.durable) / 2;

			let p_two = (vfe.current_ability.efficiency
				+ vfe.current_ability.skill
				+ vfe.current_ability.luck
				+ vfe.current_ability.durable)
				/ (4 * vfe.current_ability.durable);

			let p_two = p_two.pow(2) * vfe.level;
			let total_charge_cost = BalanceOf::<T>::from((p_one + p_two) * charge_num).saturating_mul(T::CostUnit::get());

			// try to burn the charge
			T::Currencies::burn_from(T::IncentiveToken::get(), &owner, total_charge_cost)?;

			vfe.remaining_battery = vfe.remaining_battery + charge_num;

			// save common_prize
			VFEDetails::<T>::insert(brand_id.clone(), instance.clone(), vfe);

			Self::deposit_event(Event::PowerRestored(owner, charge_num, total_charge_cost, brand_id, instance));
			Ok(())
		}

		/// restore energy
		/// - origin AccountId
		#[pallet::weight(10_000)]
		pub fn restore_energy(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let mut user = Users::<T>::get(&who).ok_or(Error::<T>::UserNotExist)?;
			ensure!(user.energy < user.energy_total, Error::<T>::UserEnergyIsFull);

			let last_restore_block = user.last_restore_block;
			let current_height = frame_system::Pallet::<T>::block_number();
			let blocks = current_height - user.last_restore_block;
			let duration = T::EnergyRecoveryDuration::get();
			let last_energy_recovery = LastEnergyRecovery::<T>::get();

			let user_energy_recovery_times = last_restore_block.checked_div(&duration).ok_or(Error::<T>::ValueOverflow)?;
			let user_last_global_energy_recovery = user_energy_recovery_times.saturating_mul(duration);
			let recoverable_times = last_energy_recovery.saturating_sub(user_last_global_energy_recovery);
			let recoverable_times = recoverable_times.checked_div(&duration).ok_or(Error::<T>::ValueOverflow)?;
			let recoverable_energy = recoverable_times.saturated_into::<u16>() * user.energy_total / 4;
			let max_recovery_energy = recoverable_energy + user.energy;
			let restored_amount = if max_recovery_energy > user.energy_total {
				user.energy = user.energy_total;
				user.energy_total - user.energy
			} else {
				user.energy = max_recovery_energy;
				recoverable_energy
			};
			Users::<T>::insert(&who, user);
			Self::deposit_event(Event::UserEnergyRestored(who, restored_amount));
			Ok(())
		}

		/// level up
		/// - origin AccountId
		/// - brand_id CollectionId
		/// - instance ItemId
		#[pallet::weight(10_000)]
		pub fn level_up(
			origin: OriginFor<T>,
			brand_id: T::CollectionId,
			item_id: T::ItemId,
			level_up: u16,
		) -> DispatchResult {
			// cost fee to level up vfe
			ensure!(level_up > 0, Error::<T>::ValueInvalid);
			VFEDetails::<T>::try_mutate(&brand_id, &item_id, |maybe_vfe| -> DispatchResult {
				let who = ensure_signed(origin.clone())?;
				let mut vfe = maybe_vfe.take().ok_or(Error::<T>::VFENotExist)?;
				let vfe_owner = Self::owner(&brand_id, &item_id).ok_or(Error::<T>::VFENotExist)?;
				ensure!(vfe_owner == who, Error::<T>::OperationIsNotAllowed);
				let user = Users::<T>::get(&who).ok_or(Error::<T>::UserNotExist)?;

				// Calculating level up fees for VFE
				let t = T::LevelUpCostFactor::get();
				let cost_unit = T::CostUnit::get();
				let base_ability = (vfe.base_ability.efficiency
					.saturating_add(vfe.base_ability.skill)
					.saturating_add(vfe.base_ability.luck)
					.saturating_sub(vfe.base_ability.durable)) / 2;
				let g = vfe.rarity.growth_points();
				let n = user.energy_total;
				let level_up_cost = base_ability + level_up * (g - 1) * n;
				let level_cost = BalanceOf::<T>::from(level_up_cost)
					.saturating_mul(t).saturating_mul(cost_unit);

				// level up should burn token
				T::Currencies::burn_from(T::IncentiveToken::get(), &who, level_cost)?;

				vfe.level = vfe.level + level_up;
				vfe.available_points = vfe.available_points + level_up * g;
				*maybe_vfe = Some(vfe);

				//todo: VFE level up requires a cooldown.
				//todo: How to increase the user's energy limit?

				Ok(())
			})
		}

		/// Increase ability
		/// - origin AccountId
		/// - brand_id CollectionId
		/// - instance ItemId
		/// - ability VFEAbility
		#[pallet::weight(10_000)]
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
				let total_ability = ability.efficiency + ability.skill + ability.luck + ability.durable;
				ensure!(total_ability <= vfe.available_points, Error::<T>::ValueInvalid);

				vfe.current_ability.efficiency = vfe.current_ability.efficiency.saturating_add(ability.efficiency);
				vfe.current_ability.skill = vfe.current_ability.skill.saturating_add(ability.skill);
				vfe.current_ability.luck = vfe.current_ability.efficiency.saturating_add(ability.luck);
				vfe.current_ability.durable = vfe.current_ability.efficiency.saturating_add(ability.durable);

				*maybe_vfe = Some(vfe);

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
			class: T::CollectionId,
			instance: T::ItemId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			let to = T::Lookup::lookup(dest.clone())?;

			let vfe = VFEDetails::<T>::get(class, instance).ok_or(Error::<T>::VFENotExist)?;

			ensure!(vfe.remaining_battery == 100, Error::<T>::VFENotFullElectric);

			ensure!(!vfe.is_upgrading, Error::<T>::VFEUpgrading);

			pallet_uniques::Pallet::<T, T::UniquesInstance>::transfer(
				origin,
				class.clone(),
				instance.clone(),
				dest,
			)?;
			Self::deposit_event(Event::Transferred(class, instance, from, to));
			Ok(())
		}

		/// sha256 for test
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn sha256_test(origin: OriginFor<T>, text: Vec<u8>) -> DispatchResult {
			let final_msg = Sha256::Hash::hash(&text).to_vec();

			Self::deposit_event(Event::Sha256Test(final_msg));

			Ok(())
		}

		/// sign for test
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn sign_test(
			origin: OriginFor<T>,
			private_key: Vec<u8>,
			msg: Vec<u8>,
		) -> DispatchResult {
			let final_msg = Sha256::Hash::hash(&msg).to_vec();

			let signing_key =
				SigningKey::from_bytes(&private_key[..]).map_err(|_| Error::<T>::SigEncodeError)?; //

			let signature = signing_key.sign(&msg[..]);

			let verifying_key = signing_key.verifying_key(); // Serialize with `::to_encoded_point()`
			let public_key: PublicKey<NistP256> = verifying_key.into();
			let encoded_point = public_key.to_encoded_point(true);

			// privatekey, pubkey, msg, sha256msg, signature

			Self::deposit_event(Event::SignTest(
				private_key,
				encoded_point.as_bytes().to_vec(),
				msg,
				final_msg,
				signature.as_bytes().to_vec(),
			));

			Ok(())
		}

		/// verify for this
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn verify_test(
			origin: OriginFor<T>,
			req_sig: Vec<u8>,
			public_key: Vec<u8>,
			msg: Vec<u8>,
		) -> DispatchResult {
			let final_msg = Sha256::Hash::hash(&msg).to_vec();

			let target = &req_sig[..];
			let sig = Signature::from_bytes(target).map_err(|_| Error::<T>::SigEncodeError)?;

			let pk = &public_key[..];
			let verify_key =
				VerifyingKey::from_sec1_bytes(pk).map_err(|_| Error::<T>::PublicKeyEncodeError)?;

			// check the validity of the signature
			// let final_msg: &[u8] = &msg.as_ref();
			let flag = verify_key.verify(&msg[..], &sig).is_ok();

			ensure!(flag, Error::<T>::OperationIsNotAllowedForSign);

			//  pubkey, msg, sha256msg, signature, isValid
			Self::deposit_event(Event::VerifyTest(public_key, msg, final_msg, req_sig, flag));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
	where
		T::CollectionId: From<T::ObjectId>,
		T::ItemId: From<T::ObjectId>,
		T::ObjectId: From<T::CollectionId>,
{
	pub fn do_mint(
		class_id: T::CollectionId,
		instance_id: T::ItemId,
		owner: T::AccountId,
	) -> DispatchResult {
		pallet_uniques::Pallet::<T, T::UniquesInstance>::mint_into(
			&class_id,
			&instance_id,
			&owner,
		)?;
		Self::deposit_event(Event::Issued(class_id, instance_id, owner));
		Ok(())
	}

	pub fn do_burn(class_id: T::CollectionId, instance_id: T::ItemId) -> DispatchResult {
		let owner = Self::owner(&class_id, &instance_id).ok_or(Error::<T>::InstanceNotFound)?;
		<pallet_uniques::Pallet<T, T::UniquesInstance> as Mutate<T::AccountId>>::burn(
			&class_id,
			&instance_id,
			None,
		)?;
		Self::deposit_event(Event::Burned(class_id, instance_id, owner));
		Ok(())
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
		for _ in 1..T::MaxGenerateRandom::get() {
			if random_number < u32::MAX - u32::MAX % (total as u32) {
				break;
			}

			let (random_number2, _, _) = Self::generate_random_number();
			random_number = random_number2;
		}

		(random_number as u16) % total
	}

	fn verify_bind_device_message(
		account: T::AccountId,
		nonce: u32,
		puk: [u8; 33],
		signature: &[u8],
	) -> Result<bool, DispatchError> {
		let pk = &puk[..];
		let verify_key =
			VerifyingKey::from_sec1_bytes(&puk[..]).map_err(|_| Error::<T>::PublicKeyEncodeError)?;

		let sig = Signature::from_bytes(signature).map_err(|_| Error::<T>::SigEncodeError)?;

		let account_nonce = nonce.to_le_bytes().to_vec();
		let account_rip160 = Ripemd::Hash::hash(account.encode().as_ref());

		let mut msg: Vec<u8> = Vec::new();
		msg.extend(account_nonce);
		msg.extend(account_rip160.to_vec());

		// check the validity of the signature
		let flag = verify_key.verify(&msg, &sig).is_ok();

		return Ok(flag);
	}

	// check the device's public key.
	fn check_device_pub(
		account: T::AccountId,
		puk: [u8; 33],
		signature: BoundedVec<u8, T::StringLimit>,
		nonce: u32,
	) -> Result<Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>, DispatchError> {
		// get the producer owner
		let mut device = Devices::<T>::get(puk.clone()).ok_or(Error::<T>::DeviceNotExist)?;

		let flag = Self::verify_bind_device_message(account, nonce.clone(), puk, &signature[..])?;

		ensure!(flag, Error::<T>::OperationIsNotAllowedForSign);

		// check the nonce
		ensure!(nonce > device.nonce, Error::<T>::NonceMustGreatThanBefore);

		device.nonce = nonce;

		Devices::<T>::insert(&puk, device);

		Ok(device)
	}

	// check the device's public key.
	fn check_device_data(
		account: T::AccountId,
		puk: [u8; 33],
		req_sig: BoundedVec<u8, T::StringLimit>,
		msg: BoundedVec<u8, T::StringLimit>,
	) -> Result<Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>, DispatchError> {
		// get the producer owner
		let device = Devices::<T>::get(puk).ok_or(Error::<T>::DeviceNotExist)?;

		let instance = device.item_id.ok_or(Error::<T>::DeviceNotBond)?;

		let device_owner =
			Self::owner(&device.brand_id, &instance).ok_or(Error::<T>::InstanceNotBelongAnyone)?;

		ensure!(account == device_owner, Error::<T>::InstanceNotBelongTheTarget);

		let target = &req_sig[..];
		let sig = Signature::from_bytes(target).map_err(|_| Error::<T>::SigEncodeError)?;

		let pk = &puk[..];
		let verify_key =
			VerifyingKey::from_sec1_bytes(pk).map_err(|_| Error::<T>::PublicKeyEncodeError)?;

		// check the validity of the signature
		let final_msg: &[u8] = &msg.as_ref();
		let flag = verify_key.verify(final_msg, &sig).is_ok();

		ensure!(flag, Error::<T>::OperationIsNotAllowedForSign);

		Ok(device)
	}

	// handler report data to get rewards
	fn handler_report_data(
		device: &mut Device<T::CollectionId, T::ItemId, T::ObjectId, AssetIdOf<T>, BalanceOf<T>>,
		account: T::AccountId,
		report_data: BoundedVec<u8, T::StringLimit>,
	) -> Result<(), DispatchError> {
		let brand_id = device.brand_id;
		let item_id = device.item_id.ok_or(Error::<T>::DeviceNotBond)?;
		let sport_type = device.sport_type;

		match sport_type {
			SportType::JumpRope => {
				ensure!(report_data.len() == 17, Error::<T>::DeviceMsgNotCanNotBeDecode);
				// let msg_vec = msg.to_vec();
				let timestamp_vec: [u8; 4] =
					report_data[0..4].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				// let mode = msg[4];
				let skipping_times_vec: [u8; 2] =
					report_data[4..6].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let skipping_duration_vec: [u8; 2] =
					report_data[6..8].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let training_count_vec: [u8; 2] =
					report_data[8..10].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let average_frequency_vec: [u8; 2] =
					report_data[10..12].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let maximum_frequency_vec: [u8; 2] =
					report_data[12..14].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let maximum_skipping_vec: [u8; 2] =
					report_data[14..16].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;
				let number_of_miss = report_data[14];
				// let effective_skipping_times_vec: [u8; 2] =
				// 	report_data[16..18].try_into().map_err(|_| Error::<T>::DeviceMsgDecodeErr)?;

				let training_time = u32::from_le_bytes(timestamp_vec);
				// let skipping_times = u16::from_le_bytes(skipping_times_vec);
				let training_duration = u16::from_le_bytes(skipping_duration_vec);
				let training_count = u16::from_le_bytes(training_count_vec);

				let average_frequency = u16::from_le_bytes(average_frequency_vec);
				// let maximum_frequency = u16::from_le_bytes(maximum_frequency_vec);
				let maximum_jumps = u16::from_le_bytes(maximum_skipping_vec);
				// let effective_skipping_times = u16::from_le_bytes(effective_skipping_times_vec);

				ensure!(
					training_time > device.timestamp,
					Error::<T>::DeviceTimeStampMustGreaterThanBefore
				);

				let mut vfe =
					VFEDetails::<T>::get(brand_id, item_id).ok_or(Error::<T>::VFENotExist)?;

				let mut user = Users::<T>::get(account.clone()).ok_or(Error::<T>::UserNotExist)?;

				// Power consumption = training-duration / training_unit_duration
				let mut power_used = training_duration / sport_type.training_unit_duration();

				// check the user energy
				if power_used > user.energy {
					power_used = user.energy;
				}

				// check the vfe electric
				if power_used > vfe.remaining_battery {
					power_used = vfe.remaining_battery;
				}

				// update user energy and vfe remaining battery
				user.energy = user.energy - power_used;
				vfe.remaining_battery = vfe.remaining_battery.clone() - power_used;

				let r_luck = Self::random_value(vfe.current_ability.luck) + 1;
				let r_skill =
					(vfe.current_ability.skill * maximum_jumps) / ((number_of_miss as u16 + 1) * sport_type.frequency_standard());
				let s = if vfe.current_ability.skill > r_skill {
					vfe.current_ability.skill - Self::random_value(vfe.current_ability.skill - r_skill)
				} else {
					vfe.current_ability.skill + Self::random_value(r_skill - vfe.current_ability.skill)
				};

				let f = sport_type.is_frequency_range(average_frequency);
				let e = vfe.current_ability.efficiency;

				let training_volume = (e + s + 2 * r_luck) * power_used * f;

				let final_award = BalanceOf::<T>::from(training_volume)
					.saturating_mul(T::CostUnit::get());

				// update the electric with user and vfe and device.
				device.timestamp = training_time;
				Devices::<T>::insert(device.pk, device);
				Users::<T>::insert(account.clone(), user);
				VFEDetails::<T>::insert(brand_id.clone(), item_id.clone(), vfe);

				let reward_asset_id = T::IncentiveToken::get();
				T::Currencies::mint_into(
					reward_asset_id,
					&account.clone(),
					final_award.clone(),
				)?;

				Self::deposit_event(Event::TrainingReportsAndRewards(
					account,
					brand_id,
					item_id,
					sport_type,
					training_time,
					training_duration,
					training_count,
					power_used,
					reward_asset_id,
					final_award));

				Ok(())
			}
			SportType::Run => Err(Error::<T>::DeviceMsgNotCanNotBeDecode)?,
			SportType::Bicycle => Err(Error::<T>::DeviceMsgNotCanNotBeDecode)?,
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
		ensure!(owner == producer.owner, Error::<T>::OperationIsNotAllowedForProducer);

		Ok(producer.into())
	}

	// check the user if it is not exist and create it
	fn create_new_user(account_id: T::AccountId) {
		// get the producer owner
		let block_number = frame_system::Pallet::<T>::block_number();
		if !Users::<T>::contains_key(account_id.clone()) {
			let user = User {
				owner: account_id.clone(),
				energy_total: 8,
				energy: 8,
				create_block: block_number,
				last_restore_block: block_number,
			};

			Users::<T>::insert(account_id, user);
		}
	}

	// create VFE
	pub fn create_vfe(
		class_id: &T::CollectionId,
		producer_id: &T::ObjectId,
		owner: &T::AccountId,
	) -> Result<T::ItemId, DispatchError> {
		let vfe_brand = VFEBrands::<T>::get(class_id).ok_or(Error::<T>::VFEBrandNotFound)?;
		let rarity = vfe_brand.rarity;

		let (min, max) = rarity.base_range_of_ability();
		let efficiency = min + Self::random_value(max - min);
		let skill = min + Self::random_value(max - min);
		let luck = min + Self::random_value(max - min);
		let durable = min + Self::random_value(max - min);
		let (_, gene, _) = Self::generate_random_number();

		// approve producer to mint new vfe
		let item_id = Self::do_mint_approved(class_id.to_owned(), producer_id, &owner)?;

		let block_number = frame_system::Pallet::<T>::block_number();
		let base_ability = VFEAbility {
			efficiency,
			skill,
			luck,
			durable,
		};
		let vfe = VFEDetail {
			class_id: class_id.to_owned(),
			instance_id: item_id.clone(),
			base_ability: base_ability.clone(),
			current_ability: base_ability,
			rarity,
			level: 0,
			remaining_battery: 100,
			gene,
			last_block: block_number,
			is_upgrading: false,
			available_points: 0,
		};

		// save vfe detail
		VFEDetails::<T>::insert(&class_id, &item_id, vfe.clone());

		Self::deposit_event(Event::VFECreated(owner.to_owned(), vfe));

		Ok(item_id)
	}

	pub fn do_approve_mint(
		brand_id: T::CollectionId,
		operator: &T::AccountId,
		producer_id: &T::ObjectId,
		mint_amount: u32,
		mint_cost: Option<(AssetIdOf<T>, BalanceOf<T>)>,
	) -> DispatchResult {

		//Check vfe brand owner
		let vfe_brand_owner = Self::collection_owner(&brand_id).ok_or(Error::<T>::VFEBrandNotFound)?;
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
				}
			};

			if mint_cost != None {
				// only total_can_mint == 0 can mutate total_can_mint
				ensure!(approved.remaining_mint == 0, Error::<T>::RemainingMintAmountIsNotZero);
				approved.mint_cost = mint_cost;
			}

			approved.remaining_mint = approved.remaining_mint.saturating_add(mint_amount);
			*maybe_approved = Some(approved);


			VFEBrands::<T>::insert(&brand_id, vfe_brand);
			Self::deposit_event(Event::ApprovedMint(
				brand_id,
				producer_id.to_owned(),
				mint_amount,
				mint_cost,
			));

			Ok(())
		})
	}

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
				let registered = approved
					.registered
					.checked_sub(One::one())
					.ok_or(Error::<T>::ValueOverflow)?;

				let activated = approved
					.activated
					.checked_add(One::one())
					.ok_or(Error::<T>::ValueOverflow)?;

				let vfe_brand_owner = Self::collection_owner(&vfe_brand_id).ok_or(Error::<T>::VFEBrandNotFound)?;

				let instance = T::UniqueId::generate_object_id(vfe_brand_id.into())?;
				Self::do_mint(vfe_brand_id.clone(), instance.into(), who.clone())?;

				// mint_cost handle transfer
				if let Some((mint_asset_id, mint_price)) = approved.mint_cost {
					// transfer tokens to NFT class owner
					<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
						mint_asset_id,
						&Self::into_account_id(producer_id.to_owned()),
						&vfe_brand_owner,
						mint_price,
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

	// level into energy
	// pub fn level_into_energy(level: u16) -> u16 {
	// 	(level / 2) * 4 + 8
	// }
}
