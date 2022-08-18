#![cfg_attr(not(feature = "std"), no_std)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency,
			 fungibles::{Inspect as MultiAssets, Transfer, Mutate as MultiAssetsMutate},
			 tokens::nonfungibles::{Create, Inspect, Mutate},
	},
	transactional,
};

use frame_system::pallet_prelude::*;
use scale_info::{
	prelude::format,
	TypeInfo,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, One, Verify, AtLeast32BitUnsigned, StaticLookup,MaybeSerializeDeserialize},
	PerU16, RuntimeDebug, SaturatedConversion,
};
use pallet_support::fungibles::AssetFronze;
use sp_std::vec::Vec;
pub use vfemart_traits::*;
mod mock;
mod tests;

pub use pallet::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Order<AssetId,Balance, BlockNumber, ClassId, InstanceId> {
	/// currency ID.
	pub asset_id: AssetId,
	/// The balances to create an order
	pub deposit: Balance,
	/// Price of this Instance.
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// vfe list
	pub items: Vec<OrderItem<ClassId, InstanceId>>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<AssetId,Balance, BlockNumber, ClassId, InstanceId> {
	/// currency ID.
	pub asset_id: AssetId,
	/// Price of this Instance.
	#[codec(compact)]
	pub price: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumber,
	/// vfe list
	pub items: Vec<OrderItem<ClassId, InstanceId>>,
	/// commission rate
	#[codec(compact)]
	pub commission_rate: PerU16,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
enum Releases {
	V1_0_0,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V1_0_0
	}
}

pub type InstanceIdOf<T> = <T as pallet::Config>::InstanceId;
pub type ClassIdOf<T> = <T as pallet::Config>::ClassId;
type BalanceOf<T> =
<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::AssetId;

pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type OrderOf<T> = Order<AssetIdOf<T>,BalanceOf<T>, BlockNumberOf<T>, ClassIdOf<T>, InstanceIdOf<T>>;
pub type OfferOf<T> = Offer<AssetIdOf<T>,BalanceOf<T>, BlockNumberOf<T>, ClassIdOf<T>, InstanceIdOf<T>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multiple asset types
		type Currencies: MultiAssets<Self::AccountId,Balance = Balance>
		+ Transfer<Self::AccountId,Balance = Balance> + MultiAssetsMutate<Self::AccountId,Balance = Balance> + AssetFronze<AssetIdOf<Self>,Self::AccountId,BalanceOf<Self>>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Identifier for the class of asset.
		type ClassId: Member + Parameter + Default + Copy + HasCompact;

		/// The type used to identify a unique asset within an asset class.
		type InstanceId: Member + Parameter + Default + Copy + HasCompact + From<u16>;

		/// VFEMart vfe
		type VFE: VfemartVfe<Self::AccountId, Self::ClassId, Self::InstanceId>;

		/// Extra Configurations
		type ExtraConfig: VfemartConfig<Self::AccountId, BlockNumberFor<Self>>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryPalletId: Get<frame_support::PalletId>;

	}

	#[pallet::error]
	pub enum Error<T> {
		/// submit with invalid deposit
		SubmitWithInvalidDeposit,
		/// submit with invalid deadline
		SubmitWithInvalidDeadline,
		// Take Expired Order or Offer
		TakeExpiredOrderOrOffer,
		/// too many Instance charged royalty
		TooManyTokenChargedRoyalty,
		/// order not found
		OrderNotFound,
		OfferNotFound,
		/// cannot take one's own order
		TakeOwnOrder,
		TakeOwnOffer,
		InvalidCommissionRate,
		SenderTakeCommission,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder(T::AccountId, GlobalId),
		/// RemovedOrder \[who, order_id\]
		RemovedOrder(T::AccountId, GlobalId),
		RemovedOffer(T::AccountId, GlobalId),
		/// TakenOrder \[purchaser, order_owner, order_id\]
		TakenOrder(
			T::AccountId,
			T::AccountId,
			GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// TakenOrder \[token_owner, offer_owner, order_id\]
		TakenOffer(
			T::AccountId,
			T::AccountId,
			GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// CreatedOffer \[who, order_id\]
		CreatedOffer(T::AccountId, GlobalId),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			0
		}

		fn integrity_test() {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StorageVersion<T>>::put(Releases::default());
		}
	}

	/// Storage version of the pallet.
	#[pallet::storage]
	pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	// /// Index/store orders by Instance as primary key and order id as secondary key.
	// #[pallet::storage]
	// #[pallet::getter(fn order_by_token)]
	// pub type OrderByToken<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (ClassIdOf<T>, InstanceIdOf<T>), Twox64Concat, OrderIdOf<T>, T::AccountId>;

	/// Index/store orders by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> =
	StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, OrderOf<T>>;


	/// Index/store offers by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> =
	StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, OfferOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an order.
		///
		/// - `asset_id`: currency id
		/// - `category_id`: category id
		/// - `deposit`: The balances to create an order
		/// - `price`: vfes' price.
		/// - `deadline`: deadline
		/// - `items`: a list of `(class_id, instance_id, quantity, price)`
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, InstanceIdOf<T>, InstanceIdOf<T>)>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			ensure!(
				deposit >= BalanceOf::<T>::from(T::ExtraConfig::get_min_order_deposit()),
				Error::<T>::SubmitWithInvalidDeposit
			);


			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);
			let mut order = Order {
				asset_id,
				deposit,
				price,
				deadline,
				items: Vec::with_capacity(items.len()),
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::VFE>(Some(&who), &items, &mut order.items)?;

			let order_id = T::ExtraConfig::get_then_inc_id()?;
			Orders::<T>::insert(&who, order_id, order);
			Self::deposit_event(Event::CreatedOrder(who, order_id));
			Ok(().into())
		}

		/// Take a VFE order.
		///
		/// - `order_id`: order id
		/// - `order_owner`: Instance owner
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: GlobalId,
			order_owner: <T::Lookup as StaticLookup>::Source,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			let order_owner = T::Lookup::lookup(order_owner)?;

			// Simplify the logic, to make life easier.
			ensure!(purchaser != order_owner, Error::<T>::TakeOwnOrder);

			if let Some(c) = &commission_agent {
				ensure!(&purchaser != c, Error::<T>::SenderTakeCommission);
			}

			let order: OrderOf<T> = Self::delete_order(&order_owner, order_id)?;

			// Skip check deadline of orders
			// Orders are supposed to be valid until taken or cancelled

			let (items, commission_agent) = to_item_vec!(order, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::Currencies, T::VFE, _, _, _, _>(
				&purchaser,
				&order_owner,
				order.asset_id,
				order.price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::TakenOrder(
				purchaser,
				order_owner,
				order_id,
				commission_agent,
				commission_data,
			));
			Ok(().into())
		}

		/// remove an order by order owner.
		///
		/// - `order_id`: order id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::delete_order(&who, order_id)?;
			Self::deposit_event(Event::RemovedOrder(who, order_id));
			Ok(().into())
		}

		/// remove an offer by offer owner.
		///
		/// - `offer_id`: offer id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_offer(
			origin: OriginFor<T>,
			#[pallet::compact] offer_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::delete_offer(&who, offer_id)?;
			Self::deposit_event(Event::RemovedOffer(who, offer_id));
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_offer(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, InstanceIdOf<T>, InstanceIdOf<T>)>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);

			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			// Reserve balances of `asset_id` for tokenOwner to accept this offer.
			T::Currencies::frozen_balance(&purchaser,asset_id, price)?;

			let mut offer = Offer {
				asset_id,
				price,
				deadline,
				items: Vec::with_capacity(items.len()),
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::VFE>(None, &items, &mut offer.items)?;

			let offer_id = T::ExtraConfig::get_then_inc_id()?;
			Offers::<T>::insert(&purchaser, offer_id, offer);
			Self::deposit_event(Event::CreatedOffer(purchaser, offer_id));
			Ok(().into())
		}

		/// Take a VFE offer.
		///
		/// - `offer_id`: offer id
		/// - `offer_owner`: Instance owner
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_offer(
			origin: OriginFor<T>,
			#[pallet::compact] offer_id: GlobalId,
			offer_owner: <T::Lookup as StaticLookup>::Source,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let token_owner = ensure_signed(origin)?;
			let offer_owner = T::Lookup::lookup(offer_owner)?;

			// Simplify the logic, to make life easier.
			ensure!(offer_owner != token_owner, Error::<T>::TakeOwnOffer);

			if let Some(c) = &commission_agent {
				ensure!(&token_owner != c, Error::<T>::SenderTakeCommission);
			}

			let offer: OfferOf<T> = Self::delete_offer(&offer_owner, offer_id)?;

			// Check deadline of this offer
			ensure!(
				frame_system::Pallet::<T>::block_number() < offer.deadline,
				Error::<T>::TakeExpiredOrderOrOffer
			);

			let (items, commission_agent) = to_item_vec!(offer, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::Currencies, T::VFE, _, _, _, _>(
				&offer_owner,
				&token_owner,
				offer.asset_id,
				offer.price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::TakenOffer(
				token_owner,
				offer_owner,
				offer_id,
				commission_agent,
				commission_data,
			));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn delete_order(who: &T::AccountId, order_id: GlobalId) -> Result<OrderOf<T>, DispatchError> {
		Orders::<T>::try_mutate_exists(who, order_id, |maybe_order| {
			let order: OrderOf<T> = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?.clone();

			// Can we safely ignore this remain value?
			let _remain: BalanceOf<T> =
				T::Currencies::unfrozen_balance(who, order.asset_id,order.deposit.saturated_into())?;

			for item in &order.items {
				T::VFE::unreserve_tokens(who, item.class_id, item.instance_id, item.quantity)?;
			}

			*maybe_order = None;
			Ok(order)
		})
	}

	fn delete_offer(who: &T::AccountId, order_id: GlobalId) -> Result<OfferOf<T>, DispatchError> {
		Offers::<T>::try_mutate_exists(who, order_id, |maybe_offer| {
			let offer: OfferOf<T> = maybe_offer.as_mut().ok_or(Error::<T>::OfferNotFound)?.clone();

			// Can we safely ignore this remain value?
			let _remain: BalanceOf<T> = T::Currencies::unfrozen_balance(who,offer.asset_id, offer.price.saturated_into())?;

			*maybe_offer = None;

			Ok(offer)

		})
	}

	pub fn treasury_account_id() -> T::AccountId {
		sp_runtime::traits::AccountIdConversion::<T::AccountId>::into_account(
			&T::TreasuryPalletId::get(),
		)
	}
}

impl<T: Config> vfemart_traits::VfemartOrder<T::AccountId, ClassIdOf<T>, InstanceIdOf<T>>
for Pallet<T>
{
	fn burn_orders(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		instance_id: InstanceIdOf<T>,
	) -> DispatchResult {
		let all_orders: Vec<GlobalId> = Orders::<T>::iter()
			.filter(|(owner, _order_id, order)| {
				let items = &order.items;
				owner == who &&
					items
						.into_iter()
						.any(|item| item.class_id == class_id && item.instance_id == instance_id)
			})
			.map(|(_owner, order_id, _order)| order_id)
			.collect();
		for order_id in all_orders {
			Self::delete_order(&who, order_id)?;
			Self::deposit_event(Event::RemovedOrder(who.clone(), order_id));
		}
		Ok(())
	}
	fn burn_offers(
		who: &T::AccountId,
		class_id: ClassIdOf<T>,
		instance_id: InstanceIdOf<T>,
	) -> DispatchResult {
		let all_offers: Vec<GlobalId> = Offers::<T>::iter()
			.filter(|(owner, _offer_id, offer)| {
				let items = &offer.items;
				owner == who &&
					items
						.into_iter()
						.any(|item| item.class_id == class_id && item.instance_id == instance_id)
			})
			.map(|(_owner, offer_id, _offer)| offer_id)
			.collect();
		for offer_id in all_offers {
			Self::delete_offer(&who, offer_id)?;
			Self::deposit_event(Event::RemovedOffer(who.clone(), offer_id));
		}
		Ok(())
	}
}
