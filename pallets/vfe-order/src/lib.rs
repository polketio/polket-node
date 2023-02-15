#![cfg_attr(not(feature = "std"), no_std)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency,
			 fungibles::{Inspect as MultiAssets, Transfer, Mutate as MultiAssetsMutate},
			 tokens::nonfungibles::{
				Create, Inspect, InspectEnumerable, Mutate, Transfer as NFTTransfer,
			},
	},
	transactional,PalletId,
};
use pallet_support::uniqueid::UniqueIdGenerator;
use pallet_support::trade::UniqueTradeGenerator;
use frame_system::pallet_prelude::*;
use scale_info::{
	prelude::format,
	TypeInfo,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, One, Verify, AtLeast32BitUnsigned, StaticLookup,MaybeSerializeDeserialize,
		Saturating,CheckedSub,
	
	},
	PerU16, RuntimeDebug, SaturatedConversion,
};
use pallet_support::fungibles::AssetFronze;
use sp_std::vec::Vec;
use frame_support::traits::fungibles;
pub mod types;
// mod mock;
// mod tests;

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


pub type OrderOf<T> = Order<AssetIdOf<T>,BalanceOf<T>, BlockNumberOf<T>,CollectionIdOf<T>,ItemIdOf<T>,<T as Config>::StringLimit >;

pub type OfferOf<T> = Offer<AssetIdOf<T>,BalanceOf<T>, BlockNumberOf<T>,CollectionIdOf<T>,ItemIdOf<T>,<T as Config>::StringLimit>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multiple asset types
		type Currencies: MultiAssets<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ MultiAssetsMutate<Self::AccountId>;

		/// Identifier for the class of asset.
		type CollectionId: Member + Parameter + Default + Copy +MaxEncodedLen + HasCompact;

		/// The type used to identify a unique asset within an asset class.
		type ItemId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen + From<u16>;

		/// VFEMart vfe
		// type VFE: VfemartVfe<Self::AccountId, Self::CollectionId, Self::ItemId>;

		/// Extra Configurations
		// type ExtraConfig: VfemartConfig<Self::AccountId, BlockNumberFor<Self>>;


		/// The maximum length of data stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32> + Clone;

		/// pallet-uniques instance
		type UniquesInstance: NFTTransfer<Self::AccountId,CollectionId = Self::CollectionId,ItemId = Self::ItemId>;

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder(T::AccountId, T::ObjectId),
		/// RemovedOrder \[who, order_id\]
		RemovedOrder(T::AccountId, T::ObjectId),
		RemovedOffer(T::AccountId, T::ObjectId),
		/// TakenOrder \[purchaser, order_owner, order_id\]
		TakenOrder(
			T::AccountId,
			T::AccountId,
			T::ObjectId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// TakenOrder \[token_owner, offer_owner, order_id\]
		TakenOffer(
			T::AccountId,
			T::AccountId,
			T::ObjectId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		/// CreatedOffer \[who, order_id\]
		CreatedOffer(T::AccountId, T::ObjectId),
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
	// pub type OrderByToken<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (CollectionIdOf<T>, ItemIdOf<T>), Twox64Concat, OrderIdOf<T>, T::AccountId>;



	/// Index/store orders by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> =
	StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, T::ObjectId,  OrderOf<T>>;


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
			#[pallet::compact] deposit: BalanceOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: BoundedVec<OrderItem<T::CollectionId, T::ItemId>, T::StringLimit>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// ensure!(
			// 	commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
			// 	Error::<T>::InvalidCommissionRate
			// );

			// ensure!(
			// 	deposit >= BalanceOf::<T>::from(T::ExtraConfig::get_min_order_deposit()),
			// 	Error::<T>::SubmitWithInvalidDeposit
			// );


			// <T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);
			let mut order = Order {
				asset_id,
				deposit,
				price,
				deadline,
				items: items.clone(),
				commission_rate,
			};

			let order_id = T::UniqueId::generate_object_id(T::OrderId::get())?;

			let order_account_id = Self::into_account_id(order_id);

			// let item_vec = &items[..];

			for  item in  items{
				// VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
				T::UniquesInstance::transfer(&item.collection_id,&item.item_id,&order_account_id)?;
			}

			// ensure_one_royalty!(items);
			// reserve_and_push_tokens::<_, _, _, T::VFE>(Some(&who), &items, &mut order.items)?;


			
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
			#[pallet::compact] order_id: T::ObjectId,
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



			let order_account_id = Self::into_account_id(order_id);

			// let item_vec = &items[..];

			for  item in  order.items.clone(){
				// VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
				T::UniquesInstance::transfer(&item.collection_id,&item.item_id,&order_account_id)?;
			}
			let items_temp = &order.items[..];

			// Skip check deadline of orders
			// Orders are supposed to be valid until taken or cancelled
			// let (items, commission_agent) = to_item_vec!(order, commission_agent);
			// let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
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

			Self::deposit_event(Event::TakenOrder(
				purchaser,
				order_owner,
				order_id,
				None,
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
			#[pallet::compact] order_id: T::ObjectId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Self::delete_order(&who, order_id)?;
			let order: OrderOf<T> = Self::delete_order(&who, order_id)?;

			// let order_account_id = Self::into_account_id(order_id);

			for  item in  order.items.clone(){
				// VFE::transfer(pay_vfes, pay_currency, *class_id, *instance_id, *quantity)?;
				T::UniquesInstance::transfer(&item.collection_id,&item.item_id,&who)?;
			}
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
			#[pallet::compact] offer_id: T::ObjectId,
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
			items: BoundedVec<OrderItem<T::CollectionId, T::ItemId>, T::StringLimit>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);

			// ensure!(
			// 	commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
			// 	Error::<T>::InvalidCommissionRate
			// );

			let offer_id  = T::UniqueId::generate_object_id(T::OfferId::get())?;

			// Reserve balances of `asset_id` for tokenOwner to accept this offer.
			// T::Currencies::frozen_balance(&purchaser,asset_id, price)?;


			<T::Currencies as fungibles::Transfer<T::AccountId>>::transfer(
				asset_id,
				&purchaser,
				&Self::into_account_id(offer_id),
				price,
				true,
			)?;


			let  offer = Offer {
				asset_id,
				price,
				deadline,
				items: items,
				commission_rate,
			};

			// ensure_one_royalty!(items);
			// reserve_and_push_tokens::<_, _, _, T::VFE>(None, &items, &mut offer.items)?;

			
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
			#[pallet::compact] offer_id: T::ObjectId,
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

			// let (items, commission_agent) = to_item_vec!(offer, commission_agent);
			// let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			// swap_assets::<T::Currencies, T::VFE, _, _, _, _>(
			// 	&offer_owner,
			// 	&token_owner,
			// 	offer.asset_id,
			// 	offer.price,
			// 	&items,
			// 	&Self::treasury_account_id(),
			// 	T::ExtraConfig::get_platform_fee_rate(),
			// 	&beneficiary,
			// 	royalty_rate,
			// 	&commission_agent,
			// )?;

			// Self::deposit_event(Event::TakenOffer(
			// 	token_owner,
			// 	offer_owner,
			// 	offer_id,
			// 	commission_agent,
			// 	commission_data,
			// ));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn delete_order(who: &T::AccountId, order_id: T::ObjectId) -> Result<OrderOf<T>, DispatchError> {
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

	fn delete_offer(who: &T::AccountId, order_id: T::ObjectId) -> Result<OfferOf<T>, DispatchError> {
		Offers::<T>::try_mutate_exists(who, order_id, |maybe_offer| {
			let offer: OfferOf<T> = maybe_offer.as_mut().ok_or(Error::<T>::OfferNotFound)?.clone();

			// Can we safely ignore this remain value?
			// let _remain: BalanceOf<T> = T::Currencies::unfrozen_balance(who,offer.asset_id, offer.price.saturated_into())?;

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
			T::UniquesInstance::transfer(&item.collection_id,&item.item_id,pay_currency)?;
		}
		Ok(())
	}



	// pub fn treasury_account_id() -> T::AccountId {
		// sp_runtime::traits::AccountIdConversion::<T::AccountId>::into_account(
		// 	&T::TreasuryPalletId::get(),
		// )
	// }
}

// impl<T: Config> vfemart_traits::VfemartOrder<T::AccountId, CollectionIdOf<T>, ItemIdOf<T>>
// for Pallet<T>
// {
// 	fn burn_orders(
// 		who: &T::AccountId,
// 		class_id: CollectionIdOf<T>,
// 		instance_id: ItemIdOf<T>,
// 	) -> DispatchResult {
// 		let all_orders: Vec<T::ObjectId> = Orders::<T>::iter()
// 			.filter(|(owner, _order_id, order)| {
// 				let items = &order.items;
// 				owner == who &&
// 					items
// 						.into_iter()
// 						.any(|item| item.class_id == class_id && item.instance_id == instance_id)
// 			})
// 			.map(|(_owner, order_id, _order)| order_id)
// 			.collect();
// 		for order_id in all_orders {
// 			Self::delete_order(&who, order_id)?;
// 			Self::deposit_event(Event::RemovedOrder(who.clone(), order_id));
// 		}
// 		Ok(())
// 	}
// 	fn burn_offers(
// 		who: &T::AccountId,
// 		class_id: CollectionIdOf<T>,
// 		instance_id: ItemIdOf<T>,
// 	) -> DispatchResult {
// 		let all_offers: Vec<T::ObjectId> = Offers::<T>::iter()
// 			.filter(|(owner, _offer_id, offer)| {
// 				let items = &offer.items;
// 				owner == who &&
// 					items
// 						.into_iter()
// 						.any(|item| item.class_id == class_id && item.instance_id == instance_id)
// 			})
// 			.map(|(_owner, offer_id, _offer)| offer_id)
// 			.collect();
// 		for offer_id in all_offers {
// 			Self::delete_offer(&who, offer_id)?;
// 			Self::deposit_event(Event::RemovedOffer(who.clone(), offer_id));
// 		}
// 		Ok(())
// 	}
// }
