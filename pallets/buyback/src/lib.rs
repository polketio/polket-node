// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles,
		fungibles::{Inspect as MultiAssets, Mutate as MultiAssetsMutate, Transfer},
		tokens::WithdrawConsequence,
	},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;

pub use pallet::*;
use pallet_support::uniqueid::UniqueIdGenerator;
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedDiv, Saturating, Zero};
use sp_std::prelude::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;

type BalanceOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::AssetId;
type PlanInfoOf<T> = PlanInfo<
	<T as frame_system::Config>::AccountId,
	AssetIdOf<T>,
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
>;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::traits::tokens::DepositConsequence;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		///  The origin which who can create buyback plan.
		type BuybackOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		///  The origin which who can participant buyback plan.
		type ParticipantOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// Multiple Asset hander, which should implement `frame_support::traits::fungibles`
		type Currencies: MultiAssets<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ MultiAssetsMutate<Self::AccountId>;

		/// Unify the value types of AssetId
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ParentId = Self::Hash, ObjectId = Self::ObjectId>;

		/// The buyback plan-id parent key
		#[pallet::constant]
		type PlanId: Get<Self::Hash>;

		/// The maximum number of iterations when processing an array.
		#[pallet::constant]
		type IterationsLimit: Get<u32>;

		/// The pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type MaxPlans: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_total_plans_count)]
	/// Self-incrementing nonce to obtain non-repeating random seeds
	pub type TotalPlansCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_buyback_plans)]
	/// Records the buyback plans.
	pub(crate) type BuybackPlans<T: Config> =
		StorageMap<_, Twox64Concat, T::ObjectId, PlanInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_participant_registrations)]
	/// Record the amount locked by those participating in the buyback plan.
	pub(crate) type ParticipantRegistrations<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::ObjectId,
		Twox64Concat,
		T::AccountId,
		ParticipantInfo<BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A buyback `PlanInfo` was created.
		PlanCreated { plan_id: T::ObjectId, plan_info: PlanInfoOf<T> },

		/// A buyback `PlanInfo` was canceled.
		PlanCanceled { plan_id: T::ObjectId },

		/// A buyback `PlanInfo` was started.
		PlanStarted { plan_id: T::ObjectId },

		/// A buyback `PlanInfo` was completed
		PlanCompleted { plan_id: T::ObjectId },

		/// A buyback `PlanInfo` was cleared
		PlanCleared { plan_id: T::ObjectId },

		/// A seller was registered the buyback plan and locked `amount` of the assets.
		SellerRegistered { plan_id: T::ObjectId, who: T::AccountId, locked: BalanceOf<T> },

		/// A user has withdrew rewards from a completed buyback `PlanInfo`.
		Withdrew { who: T::AccountId, plan_id: T::ObjectId, rewards: BalanceOf<T> },

		/// The rewards of completed `PlanInfo` have been partially returned.
		PartiallyPaybacked { plan_id: T::ObjectId },

		/// The rewards of completed `PlanInfo` have been all returned.
		AllPaybacked { plan_id: T::ObjectId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// OperationIsNotAllowed
		OperationIsNotAllowed,
		/// ValueInvalid
		ValueInvalid,
		/// RoleInvalid
		RoleInvalid,
		/// Error names should be descriptive.
		NoneValue,
		/// ValueOverflow
		ValueOverflow,
		/// asset unavailable
		AssetUnavailable,
		/// plan is not existed
		BuybackPlanNotExisted,
		/// The planned start block is greater than the current block.
		PlanStartGreaterThanCurrent,
		/// The total number of current plans has reached the maximum.
		TotalPlansReachedMax,
		/// The locked amount is less than the minimum value.
		LockedAmountLessThanMin,
		/// Insufficient balance of assets
		InsufficientBalance,
		/// The seller number of current plans has reached the maximum.
		SellersReachedMax,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			// handle `PlanInfo` status
			let mut should_clear_plans = false;
			let total_plans = TotalPlansCount::<T>::get();
			if total_plans >= T::MaxPlans::get() {
				should_clear_plans = true;
			}
			for (plan_id, plan) in BuybackPlans::<T>::iter() {
				let _ = match plan.status {
					PlanStatus::Upcoming => {
						if plan.start <= n {
							let mut plan = plan;
							plan.status = PlanStatus::InProgress;
							BuybackPlans::<T>::insert(plan_id, plan);
							Self::deposit_event(Event::<T>::PlanStarted { plan_id });
						}
						Ok(())
					},
					PlanStatus::InProgress => {
						if plan.start + plan.period <= n {
							let mut plan = plan;
							plan.status = PlanStatus::Completed;
							BuybackPlans::<T>::insert(plan_id, plan);
							Self::deposit_event(Event::<T>::PlanCompleted { plan_id });
						}
						Ok(())
					},
					PlanStatus::Completed => Self::do_payback(plan_id),
					PlanStatus::AllPaybacked =>
						if should_clear_plans {
							Self::clear_paybacked_plan(plan_id)
						} else {
							Ok(())
						},
				};
			}
			0
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a buyback plan, the operator is generally `sudo` or `council`
		/// - origin BuybackOrigin
		/// - sell_asset_id AssetId
		/// - buy_asset_id AssetId
		/// - min_sell Balance
		/// - buyback_amount Balance
		/// - start Blocknumber
		/// - period Blocknumber
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn create_plan(
			origin: OriginFor<T>,
			sell_asset_id: AssetIdOf<T>,
			buy_asset_id: AssetIdOf<T>,
			#[pallet::compact] min_sell: BalanceOf<T>,
			#[pallet::compact] buyback_amount: BalanceOf<T>,
			#[pallet::compact] seller_limit: u32,
			#[pallet::compact] start: T::BlockNumber,
			#[pallet::compact] period: T::BlockNumber,
			mode: BuybackMode,
		) -> DispatchResult {
			// 1. check if origin is `BuybackOrigin`.
			let creator = T::BuybackOrigin::ensure_origin(origin.clone())?;
			let total_count = TotalPlansCount::<T>::get();
			ensure!(total_count < T::MaxPlans::get(), Error::<T>::TotalPlansReachedMax);

			// auto increase ID
			let plan_id = T::UniqueId::generate_object_id(T::PlanId::get())?;

			ensure!(!buyback_amount.is_zero(), Error::<T>::ValueInvalid);

			// 2. Check if `sell_asset_id`  and `buy_asset_id` is existed.
			let can_deposit = T::Currencies::can_deposit(
				sell_asset_id,
				&Self::into_account_id(plan_id),
				min_sell,
				false,
			);
			ensure!(can_deposit == DepositConsequence::Success, Error::<T>::AssetUnavailable);
			let can_withdraw = T::Currencies::can_withdraw(buy_asset_id, &creator, buyback_amount);
			ensure!(can_withdraw == WithdrawConsequence::Success, Error::<T>::InsufficientBalance);

			// 3. Check if `start` block number greater than current block number.
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(start > block_number, Error::<T>::PlanStartGreaterThanCurrent);

			// 4. Transfer `buy_asset_id` to `plan_account_id` from `creator`.
			<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
				buy_asset_id,
				&creator,
				&Self::into_account_id(plan_id),
				buyback_amount,
				true,
			)?;

			// 5. Generate Plan ID and save new `PlanInfo` in `BuybackPlans`.
			let plan_info = PlanInfo {
				buy_asset_id,
				sell_asset_id,
				seller_amount: 0u32,
				min_sell,
				seller_limit,
				start,
				period,
				creator,
				status: PlanStatus::Upcoming,
				mode,
				total_buy: buyback_amount,
				total_sell: Zero::zero(),
			};
			BuybackPlans::<T>::insert(plan_id, plan_info.clone());

			TotalPlansCount::<T>::put(total_count.saturating_add(1));

			// 6. Emit Event.
			Self::deposit_event(Event::PlanCreated { plan_id, plan_info });

			Ok(())
		}

		/// cancel buyback plan
		/// - origin BuybackOrigin
		/// - plan_id u64
		#[pallet::weight(10_000)]
		pub fn cancel_plan(
			origin: OriginFor<T>,
			#[pallet::compact] plan_id: T::ObjectId,
		) -> DispatchResult {
			let who = T::BuybackOrigin::ensure_origin(origin.clone())?;
			BuybackPlans::<T>::try_mutate_exists(plan_id, |maybe_plan| -> DispatchResult {
				// 1. check if `plan_id` existed.
				let plan = maybe_plan.take().ok_or(Error::<T>::BuybackPlanNotExisted)?;

				// 2. check if plan status` is `upcoming`. Only `upcoming` plan can be canceled.
				ensure!(plan.status == PlanStatus::Upcoming, Error::<T>::OperationIsNotAllowed);

				// 3. check if origin is the creator of this `plan_id`.
				ensure!(who == plan.creator, Error::<T>::OperationIsNotAllowed);

				// 4. Return buyback asset of this plan to creator
				<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
					plan.buy_asset_id,
					&Self::into_account_id(plan_id),
					&plan.creator,
					plan.total_buy,
					false,
				)?;

				*maybe_plan = None;

				let total_count = TotalPlansCount::<T>::get();
				TotalPlansCount::<T>::put(total_count.saturating_sub(1));

				// 4. Emit Event.
				Self::deposit_event(Event::PlanCanceled { plan_id });
				Ok(())
			})
		}

		/// The seller selects the buyback plan to register the locking quantity,
		/// and the locking quantity is not less than `min_sell`
		/// - origin AccountId
		/// - plan_id u64
		/// - amount Balance
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn seller_register(
			origin: OriginFor<T>,
			#[pallet::compact] plan_id: T::ObjectId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = T::ParticipantOrigin::ensure_origin(origin.clone())?;
			BuybackPlans::<T>::try_mutate(plan_id, |maybe_plan| -> DispatchResult {
				// 1. check if `plan_id` existed.
				let mut plan = maybe_plan.take().ok_or(Error::<T>::BuybackPlanNotExisted)?;
				// 2. check if plan status` is `InProgress`. Only `InProgress` plan can be
				// participated.
				ensure!(plan.status == PlanStatus::InProgress, Error::<T>::OperationIsNotAllowed);
				ensure!(plan.seller_amount < plan.seller_limit, Error::<T>::SellersReachedMax);
				// 3. Check if the amount locked by the participant is greater than the
				// minimum amount.
				ensure!(amount > plan.min_sell, Error::<T>::LockedAmountLessThanMin);

				// 4. Transfer `sell_asset_id` to `plan_account_id` from `origin`.
				<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
					plan.sell_asset_id,
					&who,
					&Self::into_account_id(plan_id),
					amount,
					true,
				)?;

				// 5. Accumulatively participant info and insert data into
				// `ParticipantRegistrations`.
				let mut participant_info = ParticipantRegistrations::<T>::get(&plan_id, &who);
				participant_info.locked = participant_info.locked.saturating_add(amount);
				ParticipantRegistrations::<T>::insert(&plan_id, &who, participant_info);

				// 6. Update `PlanInfo`.
				plan.total_sell += amount;
				plan.seller_amount += 1;
				*maybe_plan = Some(plan);

				// 7. Emit Event.
				Self::deposit_event(Event::SellerRegistered { plan_id, who, locked: amount });

				Ok(())
			})
		}

		/// Redemption of assets to completed/cancelled buyback plan
		/// - origin AccountId
		/// - who AccountId
		/// - plan_id u64
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn withdraw(
			origin: OriginFor<T>,
			who: T::AccountId,
			#[pallet::compact] plan_id: T::ObjectId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_withdraw(who, plan_id)
		}

		/// Automatic payback of assets on completed/cancelled buyback plan.
		/// It needs to be executed multiple times to complete the refund of all users,
		/// and each time the refund is executed according to the number of `IterationsLimit` users.
		/// - origin AccountId
		/// - plan_id u64
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn payback(
			origin: OriginFor<T>,
			#[pallet::compact] plan_id: T::ObjectId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_payback(plan_id)
		}

		// /// calculate
		// #[pallet::weight(10_000)]
		// pub fn calculate(origin: OriginFor<T>, num: u128) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	let num_pow = num.pow(10);
		// 	let num_nth: u128 = num_pow.nth_root(9);
		// 	Self::deposit_event(Event::SomethingStored(num_pow, num_nth, who));
		// 	Ok(())
		// }
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the Producer.
	fn into_account_id(id: T::ObjectId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(id)
	}

	fn do_withdraw(who: T::AccountId, plan_id: T::ObjectId) -> DispatchResult {
		// 1. check if `plan_id` existed.
		let plan = BuybackPlans::<T>::get(plan_id).ok_or(Error::<T>::BuybackPlanNotExisted)?;

		// 2. check if plan status` is `Completed`. Only `Completed` plan can be withdrew.
		ensure!(plan.status == PlanStatus::Completed, Error::<T>::OperationIsNotAllowed);

		// 3. Check if `origin` in `ParticipantRegistrations` of this plan.
		let mut participant_info = ParticipantRegistrations::<T>::get(&plan_id, &who);
		ensure!(!participant_info.locked.is_zero(), Error::<T>::OperationIsNotAllowed);
		// 4. Transfer `buy_asset_id` to `who` from `plan_account_id`.
		let rewards = participant_info
			.locked
			.saturating_mul(plan.total_buy)
			.checked_div(&plan.total_sell)
			.ok_or(Error::<T>::ValueInvalid)?;
		<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
			plan.buy_asset_id,
			&Self::into_account_id(plan_id),
			&who,
			rewards,
			false,
		)?;
		participant_info.rewards = rewards;
		participant_info.withdrew = true;
		// 5. Record rewards of participant.
		ParticipantRegistrations::<T>::insert(&plan_id, &who, participant_info);

		// 6. Emit Event.
		Self::deposit_event(Event::Withdrew { who, plan_id, rewards });
		Ok(())
	}

	fn do_payback(plan_id: T::ObjectId) -> DispatchResult {
		// 1. check if `plan_id` existed.
		let mut plan = BuybackPlans::<T>::get(plan_id).ok_or(Error::<T>::BuybackPlanNotExisted)?;

		// 2. check if plan status` is `Completed`. Only `Completed` plan can be withdrew.
		ensure!(plan.status == PlanStatus::Completed, Error::<T>::OperationIsNotAllowed);

		// 3. Iteration `ParticipantRegistrations` within `IterationsLimit` to `withdraw`
		// rewards.
		let participants = ParticipantRegistrations::<T>::iter_key_prefix(&plan_id);
		// let mut payback_count = 0u32;
		let mut all_payback = true;
		for (payback_count, who) in participants.enumerate() {
			if payback_count >= T::IterationsLimit::get() as usize {
				// Not everyone was able to be payback this time around.
				all_payback = false;
				break
			}
			Self::do_withdraw(who, plan_id)?;
			// payback_count += 1;
		}

		// 4. Emit Event.
		if all_payback {
			// finally handle buyback asset.
			Self::handle_buyback_asset(plan_id, plan.clone())?;

			Self::deposit_event(Event::<T>::AllPaybacked { plan_id });
			plan.status = PlanStatus::AllPaybacked;
			BuybackPlans::<T>::insert(plan_id, plan);
		} else {
			Self::deposit_event(Event::<T>::PartiallyPaybacked { plan_id });
		}
		Ok(())
	}

	fn handle_buyback_asset(plan_id: T::ObjectId, plan: PlanInfoOf<T>) -> DispatchResult {
		ensure!(plan.status == PlanStatus::Completed, Error::<T>::OperationIsNotAllowed);

		// if plan still has buy-assets, transfer the remainder to the creator.
		let remaining_buy_assets =
			T::Currencies::balance(plan.buy_asset_id, &Self::into_account_id(plan_id));
		if !remaining_buy_assets.is_zero() {
			<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
				plan.buy_asset_id,
				&Self::into_account_id(plan_id),
				&plan.creator,
				remaining_buy_assets,
				false,
			)?;
		}

		//Transfer or burn asset according to the `BuybackMode` of plan.
		match plan.mode {
			BuybackMode::Transfer =>
				<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
					plan.sell_asset_id,
					&Self::into_account_id(plan_id),
					&plan.creator,
					plan.total_sell,
					false,
				),
			BuybackMode::Burn => <T::Currencies as fungibles::Mutate<T::AccountId>>::burn_from(
				plan.sell_asset_id,
				&Self::into_account_id(plan_id),
				plan.total_sell,
			),
		}?;
		Ok(())
	}

	fn clear_paybacked_plan(plan_id: T::ObjectId) -> DispatchResult {
		BuybackPlans::<T>::try_mutate_exists(plan_id, |maybe_plan| -> DispatchResult {
			// 1. check if `plan_id` existed.
			let plan = maybe_plan.take().ok_or(Error::<T>::BuybackPlanNotExisted)?;

			// 2. check if plan status` is `upcoming`. Only `upcoming` plan can be canceled.
			ensure!(plan.status == PlanStatus::AllPaybacked, Error::<T>::OperationIsNotAllowed);

			*maybe_plan = None;
			let _ = ParticipantRegistrations::<T>::clear_prefix(plan_id, plan.seller_limit, None);

			Self::deposit_event(Event::<T>::PlanCleared { plan_id });

			let total_count = TotalPlansCount::<T>::get();
			TotalPlansCount::<T>::put(total_count.saturating_sub(1));

			Ok(())
		})
	}
}
