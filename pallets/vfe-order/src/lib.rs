#![cfg_attr(not(feature = "std"), no_std)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles,
		fungibles::{Inspect as MultiAssets, Mutate as MultiAssetsMutate, Transfer},
		tokens::nonfungibles::{
			Create, Inspect, InspectEnumerable, Mutate, Transfer as NFTTransfer,
		},
		Currency, ReservableCurrency,
	},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use pallet_support::{
	fungibles::AssetFronze, trade::UniqueTradeGenerator, uniqueid::UniqueIdGenerator,
};
use scale_info::{prelude::format, TypeInfo};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedSub,
		MaybeSerializeDeserialize, One, Saturating, StaticLookup, Verify,
	},
	PerU16, RuntimeDebug, SaturatedConversion,
};
use sp_std::vec::Vec;
pub mod types;
// mod mock;
// mod tests;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::*;

// #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
// enum Releases {
// 	V1_0_0,
// }

// impl Default for Releases {
// 	fn default() -> Self {
// 		Releases::V1_0_0
// 	}
// }

pub type ItemIdOf<T> = <T as Config>::ItemId;
pub type CollectionIdOf<T> = <T as Config>::CollectionId;
type BalanceOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
	<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::AssetId;

pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

pub type OrderOf<T> = Order<
	AssetIdOf<T>,
	BalanceOf<T>,
	BlockNumberOf<T>,
	BoundedVec<OrderItem<CollectionIdOf<T>, ItemIdOf<T>>, <T as Config>::StringLimit>,
>;

pub type OfferOf<T> =
	Offer<AssetIdOf<T>, BalanceOf<T>, BlockNumberOf<T>, OrderItem<CollectionIdOf<T>, ItemIdOf<T>>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		///  The origin which who can create order.
		type OrderOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// Multiple asset types
		type Currencies: MultiAssets<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ MultiAssetsMutate<Self::AccountId>;

		/// Identifier for the class of asset.
		type CollectionId: Member + Parameter + Default + Copy + MaxEncodedLen + HasCompact;

		/// The type used to identify a unique asset within an asset class.
		type ItemId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen + From<u16>;

		/// VFEMart vfe
		// type VFE: VfemartVfe<Self::AccountId, Self::CollectionId, Self::ItemId>;

		/// Extra Configurations
		// type ExtraConfig: VfemartConfig<Self::AccountId, BlockNumberFor<Self>>;

		/// The maximum length of data stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// pallet-uniques instance
		type UniquesInstance: NFTTransfer<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Inspect<Self::AccountId>;

		/// The pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Unify the value types of ProudcerId, CollectionId, ItemId, AssetId
		type ObjectId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The offer-id parent key
		#[pallet::constant]
		type OfferId: Get<Self::Hash>;

		/// The order-id parent key
		#[pallet::constant]
		type OrderId: Get<Self::Hash>;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<ParentId = Self::Hash, ObjectId = Self::ObjectId>;
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

		NotBelongToyYou,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder { who: T::AccountId, order_id: T::ObjectId },
		/// RemovedOrder \[who, order_id\]
		RemovedOrder { who: T::AccountId, order_id: T::ObjectId },
		/// TakenOrder \[purchaser, order_owner, order_id\]
		TakenOrder { purchaser: T::AccountId, order_owner: T::AccountId, order_id: T::ObjectId },
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
			// <StorageVersion<T>>::put(Releases::default());
		}
	}

	/// Storage version of the pallet.
	// #[pallet::storage]
	// pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	// /// Index/store orders by Instance as primary key and order id as secondary key.
	// #[pallet::storage]
	// #[pallet::getter(fn order_by_token)]
	// pub type OrderByToken<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (CollectionIdOf<T>,
	// ItemIdOf<T>), Twox64Concat, OrderIdOf<T>, T::AccountId>;

	/// Index/store orders by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, T::ObjectId, OrderOf<T>>;

	/// Index/store offers by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, T::ObjectId, OfferOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	/// Self-incrementing nonce to obtain non-repeating random seeds
	pub type OrderId<T> = StorageValue<_, u8, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an order.
		///
		/// - `origin`: origin
		/// - `asset_id`: currency id
		/// - `price`: vfes' price.
		/// - `deadline`: deadline
		/// - `items`: a list of `(class_id, instance_id, quantity, price)`
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: BoundedVec<OrderItem<T::CollectionId, T::ItemId>, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);
			let order = Order { asset_id, price, deadline, items: items.clone() };

			let order_id = T::UniqueId::generate_object_id(T::OrderId::get())?;

			let order_account_id = Self::into_account_id(order_id);

			// let item_vec = &items[..];

			for item in items {
				let owner = T::UniquesInstance::owner(&item.collection_id, &item.item_id)
					.ok_or(Error::<T>::NotBelongToyYou)?;

				ensure!(owner == who, Error::<T>::NotBelongToyYou);

				T::UniquesInstance::transfer(
					&item.collection_id,
					&item.item_id,
					&order_account_id,
				)?;
			}

			Orders::<T>::insert(&who, order_id, order);
			Self::deposit_event(Event::CreatedOrder { who, order_id });
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
			#[pallet::compact] order_id: T::ObjectId,
			order_owner: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			let order_owner = T::Lookup::lookup(order_owner)?;

			// Simplify the logic, to make life easier.
			ensure!(purchaser != order_owner, Error::<T>::TakeOwnOrder);

			let order: OrderOf<T> = Self::delete_order(&order_owner, order_id)?;

			let order_account_id = Self::into_account_id(order_id);

			// let item_vec = &items[..];

			for item in order.items.clone() {
				// VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
				T::UniquesInstance::transfer(
					&item.collection_id,
					&item.item_id,
					&order_account_id,
				)?;
			}
			let items_temp = &order.items[..];

			// Skip check deadline of orders

			Self::swap_assets(
				&purchaser,
				&order_owner,
				order.asset_id,
				order.price,
				items_temp,
				&order_owner,
				PerU16::from_percent(0),
				&order_owner,
				PerU16::from_percent(0),
				&None,
			)?;

			Self::deposit_event(Event::TakenOrder { purchaser, order_owner, order_id });
			Ok(().into())
		}

		/// remove an order by order owner.
		///
		/// - `order_id`: order id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: T::ObjectId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let order: OrderOf<T> = Self::delete_order(&who, order_id)?;

			for item in order.items.clone() {
				T::UniquesInstance::transfer(&item.collection_id, &item.item_id, &who)?;
			}
			Self::deposit_event(Event::RemovedOrder { who, order_id });
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn delete_order(
		who: &T::AccountId,
		order_id: T::ObjectId,
	) -> Result<OrderOf<T>, DispatchError> {
		Orders::<T>::try_mutate_exists(who, order_id, |maybe_order| {
			let order: OrderOf<T> = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?.clone();

			// Can we safely ignore this remain value?
			// let _remain: BalanceOf<T> =
			// 	T::Currencies::unfrozen_balance(who, order.asset_id,order.deposit.saturated_into())?;

			// for item in &order.items {
			// 	T::VFE::unreserve_tokens(who, item.class_id, item.instance_id, item.quantity)?;
			// }

			*maybe_order = None;
			Ok(order)
		})
	}

	fn delete_offer(
		who: &T::AccountId,
		order_id: T::ObjectId,
	) -> Result<OfferOf<T>, DispatchError> {
		Offers::<T>::try_mutate_exists(who, order_id, |maybe_offer| {
			let offer: OfferOf<T> = maybe_offer.as_mut().ok_or(Error::<T>::OfferNotFound)?.clone();

			// Can we safely ignore this remain value?
			// let _remain: BalanceOf<T> = T::Currencies::unfrozen_balance(who,offer.asset_id,
			// offer.price.saturated_into())?;

			*maybe_offer = None;

			Ok(offer)
		})
	}

	fn get_and_increment_order_id() -> Vec<u8> {
		let nonce = OrderId::<T>::get();
		OrderId::<T>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	pub fn into_account_id(id: T::ObjectId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(id)
	}

	fn swap_assets(
		pay_currency: &T::AccountId,
		pay_vfes: &T::AccountId,
		asset_id: AssetIdOf<T>,
		price: BalanceOf<T>,
		items: &[OrderItem<T::CollectionId, T::ItemId>],
		treasury: &T::AccountId,
		platform_fee_rate: PerU16,
		beneficiary: &T::AccountId,
		royalty_rate: PerU16,
		commission_agent: &Option<(bool, T::AccountId, PerU16)>,
	) -> DispatchResult {
		let trading_fee = platform_fee_rate.mul_ceil(price);
		let royalty_fee = royalty_rate.mul_ceil(price);

		<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
			asset_id,
			pay_currency,
			&pay_vfes,
			price,
			false,
		)?;

		<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
			asset_id,
			pay_vfes,
			&treasury,
			trading_fee,
			false,
		)?;

		<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
			asset_id,
			pay_vfes,
			&beneficiary,
			royalty_fee,
			false,
		)?;

		if let Some((status, agent, rate)) = commission_agent {
			if *status {
				let r = price.saturating_sub(trading_fee).saturating_sub(royalty_fee);

				<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
					asset_id,
					pay_vfes,
					&agent,
					rate.mul_ceil(r),
					false,
				)?;
			}
		}

		for item in items {
			// VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
			T::UniquesInstance::transfer(&item.collection_id, &item.item_id, pay_currency)?;
		}
		Ok(())
	}
}
