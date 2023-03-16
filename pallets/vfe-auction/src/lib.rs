#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use frame_system::pallet_prelude::*;
use codec::HasCompact;
use pallet_support::{ uniqueid::UniqueIdGenerator};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedDiv, Saturating, StaticLookup, Zero},
	FixedPointNumber, FixedU128, PerU16, RuntimeDebug, SaturatedConversion,
};

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		Randomness,
		fungibles::{Inspect as MultiAssets, Transfer,Mutate as MultiAssetsMutate},
		tokens::nonfungibles::{Create, Inspect, Mutate},

	},
	 PalletId,
};
use bitcoin_hashes::ripemd160 as Ripemd;

use frame_system::{
	pallet_prelude::*,
	RawOrigin,
};
use num_integer::Roots;

use pallet_uniques::WeightInfo;
use scale_info::{
	prelude::format,
	TypeInfo,
};
use sp_std::{vec::Vec,boxed::Box,convert::{TryFrom, TryInto}};
use bitcoin_hashes::sha256 as Sha256;
use bitcoin_hashes::Hash as OtherHash;
use sp_runtime::{
	traits::{
		AccountIdConversion, CheckedAdd, One,
		 Verify,
	},

};
use p256::{
	NistP256,
	elliptic_curve::{
		PublicKey,sec1::ToEncodedPoint
	},
	ecdsa::{
		SigningKey, Signature,VerifyingKey,
		signature::{
			Signer,Verifier,Signature as Sig,
		},
	},
};
pub use pallet::*;

pub const MAX_TOKEN_PER_AUCTION: u32 = 100;

mod types;

type BalanceOf<T> =
<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::Balance;
type AssetIdOf<T> =
<<T as Config>::Currencies as MultiAssets<<T as frame_system::Config>::AccountId>>::AssetId;


#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multiple asset types
		type Currencies: MultiAssets<Self::AccountId>
		+ Transfer<Self::AccountId> + MultiAssetsMutate<Self::AccountId>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The class ID type
		type CollectionId: Member + Parameter + Default + Copy + HasCompact;

		/// The class ID type
		type GlobalId: Member + Parameter + Default + Copy + HasCompact;

		/// The token ID type
		type ItemId: Member + Parameter + Default + Copy + HasCompact + From<u16>;

		/// NFTMart nft
		type VFE: NftmartNft<Self::AccountId, Self::CollectionId, Self::TokenId>;

		/// Extra Configurations
		type ExtraConfig: NftmartConfig<Self::AccountId, BlockNumberFor<Self>>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryPalletId: Get<frame_support::PalletId>;

		/// UniqueId is used to generate new CollectionId or ItemId.
		type UniqueId: UniqueIdGenerator<CollectionId=Self::CollectionId, AssetId=AssetIdOf<Self>, ItemId=Self::ItemId>;

	}

	#[pallet::error]
	pub enum Error<T> {
		/// submit with invalid deposit
		SubmitWithInvalidDeposit,
		SubmitWithInvalidDeadline,
		TooManyTokenChargedRoyalty,
		InvalidHammerPrice,
		BritishAuctionNotFound,
		DutchAuctionNotFound,
		BritishAuctionBidNotFound,
		DutchAuctionBidNotFound,
		BritishAuctionClosed,
		DutchAuctionClosed,
		PriceTooLow,
		CannotRemoveAuction,
		CannotRedeemAuction,
		CannotRedeemAuctionNoBid,
		CannotRedeemAuctionUntilDeadline,
		DuplicatedBid,
		MaxPriceShouldBeGreaterThanMinPrice,
		InvalidDutchMinPrice,
		SelfBid,
		TooManyTokens,
		EmptyTokenList,
		InvalidCommissionRate,
		SenderTakeCommission,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedBritishAuction \[who, auction_id\]
		CreatedBritishAuction(T::AccountId, T::GlobalId),
		CreatedDutchAuction(T::AccountId, T::GlobalId),
		/// RemovedBritishAuction \[who, auction_id\]
		RemovedBritishAuction(T::AccountId, T::GlobalId),
		RemovedDutchAuction(T::AccountId, T::GlobalId),
		RedeemedBritishAuction(
			T::AccountId,
			T::GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		RedeemedDutchAuction(
			T::AccountId,
			T::GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
		BidBritishAuction(T::AccountId, T::GlobalId),
		BidDutchAuction(T::AccountId, T::GlobalId),
		HammerBritishAuction(
			T::AccountId,
			T::GlobalId,
			Option<(bool, T::AccountId, PerU16)>,
			Option<Vec<u8>>,
		),
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



	/// BritishAuctions
	#[pallet::storage]
	#[pallet::getter(fn british_auctions)]
	pub type BritishAuctions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		T::GlobalId,
		BritishAuctionOf<T>,
	>;

	/// BritishAuctionBids
	#[pallet::storage]
	#[pallet::getter(fn british_auction_bids)]
	pub type BritishAuctionBids<T: Config> =
	StorageMap<_, Twox64Concat, T::GlobalId, BritishAuctionBidOf<T>>;

	/// DutchAuctions
	#[pallet::storage]
	#[pallet::getter(fn dutch_auctions)]
	pub type DutchAuctions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		T::GlobalId,
		DutchAuctionOf<T>,
	>;

	/// DutchAuctionBids
	#[pallet::storage]
	#[pallet::getter(fn dutch_auction_bids)]
	pub type DutchAuctionBids<T: Config> =
	StorageMap<_, Twox64Concat, GlobalId, DutchAuctionBidOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
		#[transactional]
		#[pallet::weight(T::WeightInfo::submit_dutch_auction(items.len() as u32))]
		pub fn submit_dutch_auction(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] min_price: Balance,
			#[pallet::compact] max_price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(CollectionIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
			allow_british_auction: bool,
			#[pallet::compact] min_raise: PerU16,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let who: T::AccountId = ensure_signed(origin)?;
			ensure!(!items.is_empty(), Error::<T>::EmptyTokenList);
			ensure!(items.len() as u32 <= MAX_TOKEN_PER_AUCTION, Error::<T>::TooManyTokens);
			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			// check and reserve `deposit`
			ensure!(
				deposit >= T::ExtraConfig::get_min_order_deposit(),
				Error::<T>::SubmitWithInvalidDeposit
			);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			let created_block = frame_system::Pallet::<T>::block_number();
			// check deadline
			ensure!(created_block < deadline, Error::<T>::SubmitWithInvalidDeadline);

			// check min price and max price
			ensure!(0 < min_price, Error::<T>::InvalidDutchMinPrice);
			ensure!(min_price < max_price, Error::<T>::MaxPriceShouldBeGreaterThanMinPrice);

			let mut auction: DutchAuctionOf<T> = DutchAuction {
				currency_id,
				deposit,
				min_price,
				max_price,
				deadline,
				created_block,
				items: Vec::with_capacity(items.len()),
				allow_british_auction,
				min_raise,
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::VFE>(Some(&who), &items, &mut auction.items)?;

			// generate an auction id
			let auction_id = T::ExtraConfig::get_then_inc_id()?;

			// save auction information.
			DutchAuctions::<T>::insert(&who, auction_id, auction);

			let auction_bid: DutchAuctionBidOf<T> = DutchAuctionBid {
				last_bid_price: min_price,
				last_bid_account: None,
				last_bid_block: Zero::zero(),
				commission_agent: None,
				commission_data: None,
			};

			DutchAuctionBids::<T>::insert(auction_id, auction_bid);

			// emit event.
			Self::deposit_event(Event::CreatedDutchAuction(who, auction_id));
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		#[transactional]
		pub fn bid_dutch_auction(
			origin: OriginFor<T>,
			#[pallet::compact] price: Balance,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			// TODO Rename to bidder
			let purchaser: T::AccountId = ensure_signed(origin)?;
			let auction_owner: T::AccountId = T::Lookup::lookup(auction_owner)?;
			ensure!(purchaser != auction_owner, Error::<T>::SelfBid);

			if let Some(c) = &commission_agent {
				ensure!(&purchaser != c, Error::<T>::SenderTakeCommission);
			}

			let auction: DutchAuctionOf<T> = Self::dutch_auctions(&auction_owner, auction_id)
				.ok_or(Error::<T>::DutchAuctionNotFound)?;
			let auction_bid: DutchAuctionBidOf<T> =
				Self::dutch_auction_bids(auction_id).ok_or(Error::<T>::DutchAuctionBidNotFound)?;

			match (&auction_bid.last_bid_account, auction.allow_british_auction) {
				(None, true) => {
					// check deadline
					let current_block: BlockNumberOf<T> = frame_system::Pallet::<T>::block_number();
					ensure!(auction.deadline >= current_block, Error::<T>::DutchAuctionClosed);
					// get price
					let current_price: Balance = calc_current_price::<T>(
						auction.max_price,
						auction.min_price,
						auction.created_block,
						auction.deadline,
						current_block,
					);
					Self::save_dutch_bid(
						auction_bid,
						auction,
						current_price,
						purchaser.clone(),
						auction_id,
						commission_agent,
						commission_data,
					)?;

					Self::deposit_event(Event::BidDutchAuction(purchaser, auction_id));
				},
				(None, false) => {
					// check deadline
					let current_block: BlockNumberOf<T> = frame_system::Pallet::<T>::block_number();
					ensure!(auction.deadline >= current_block, Error::<T>::DutchAuctionClosed);
					// get price
					let current_price: Balance = calc_current_price::<T>(
						auction.max_price,
						auction.min_price,
						auction.created_block,
						auction.deadline,
						current_block,
					);
					// delete auction
					Self::delete_dutch_auction(&auction_owner, auction_id)?;
					// swap
					let (items, commission_agent) = to_item_vec!(auction, commission_agent);
					let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
					swap_assets::<T::MultiCurrency, T::VFE, _, _, _, _>(
						&purchaser,
						&auction_owner,
						auction.currency_id,
						current_price,
						&items,
						&Self::treasury_account_id(),
						T::ExtraConfig::get_platform_fee_rate(),
						&beneficiary,
						royalty_rate,
						&commission_agent,
					)?;
					Self::deposit_event(Event::RedeemedDutchAuction(
						purchaser,
						auction_id,
						commission_agent,
						commission_data,
					));
				},
				(Some(_), true) => {
					// check deadline
					ensure!(
						get_deadline::<T>(true, Zero::zero(), auction_bid.last_bid_block) >=
							frame_system::Pallet::<T>::block_number(),
						Error::<T>::DutchAuctionClosed,
					);
					Self::save_dutch_bid(
						auction_bid,
						auction,
						price,
						purchaser.clone(),
						auction_id,
						commission_agent,
						commission_data,
					)?;

					Self::deposit_event(Event::BidDutchAuction(purchaser, auction_id));
				},
				_ => return Err(Error::<T>::DutchAuctionClosed.into()),
			}
			Ok(().into())
		}

		/// redeem
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn redeem_dutch_auction(
			origin: OriginFor<T>,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let auction_owner = T::Lookup::lookup(auction_owner)?;
			let (auction, auction_bid) = Self::delete_dutch_auction(&auction_owner, auction_id)?;
			ensure!(
				get_deadline::<T>(true, Zero::zero(), auction_bid.last_bid_block) <
					frame_system::Pallet::<T>::block_number(),
				Error::<T>::CannotRedeemAuctionUntilDeadline
			);
			ensure!(auction_bid.last_bid_account.is_some(), Error::<T>::CannotRedeemAuctionNoBid);
			let purchaser = auction_bid.last_bid_account.expect("Must be Some");

			let commission_agent = auction_bid.commission_agent.clone();
			let (items, commission_agent) = to_item_vec!(auction, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::MultiCurrency, T::VFE, _, _, _, _>(
				&purchaser,
				&auction_owner,
				auction.currency_id,
				auction_bid.last_bid_price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::RedeemedDutchAuction(
				purchaser,
				auction_id,
				commission_agent,
				auction_bid.commission_data,
			));
			Ok(().into())
		}

		/// remove a dutch auction by auction owner.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_dutch_auction(
			origin: OriginFor<T>,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let (_, bid) = Self::delete_dutch_auction(&who, auction_id)?;
			ensure!(bid.last_bid_account.is_none(), Error::<T>::CannotRemoveAuction);
			Self::deposit_event(Event::RemovedDutchAuction(who, auction_id));
			Ok(().into())
		}

		/// remove an expired dutch auction by auction owner.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_expired_dutch_auction(
			origin: OriginFor<T>,
			auction_owner: T::AccountId,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let auction: DutchAuctionOf<T> = Self::dutch_auctions(&auction_owner, auction_id)
				.ok_or(Error::<T>::DutchAuctionNotFound)?;
			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(current_block >= auction.deadline, Error::<T>::CannotRemoveAuction);
			let (_, bid) = Self::delete_dutch_auction(&auction_owner, auction_id)?;
			ensure!(bid.last_bid_account.is_none(), Error::<T>::CannotRemoveAuction);
			Self::deposit_event(Event::RemovedDutchAuction(who, auction_id));
			Ok(().into())
		}

		/// Create an British auction.
		///
		/// - `currency_id`: Currency Id
		/// - `hammer_price`: If somebody offer this price, the auction will be finished. Set to zero to disable.
		/// - `min_raise`: The next price of bid should be larger than old_price * ( 1 + min_raise )
		/// - `deposit`: A higher deposit will be good for the display of the auction in the market.
		/// - `init_price`: The initial price for the auction to kick off.
		/// - `deadline`: A block number which represents the end of the auction activity.
		/// - `allow_delay`: If ture, in some cases the deadline will be extended.
		/// - `items`: Nft list.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] hammer_price: Balance,
			#[pallet::compact] min_raise: PerU16,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] init_price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			allow_delay: bool,
			items: Vec<(CollectionIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
			#[pallet::compact] commission_rate: PerU16,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				commission_rate <= T::ExtraConfig::get_max_commission_reward_rate(),
				Error::<T>::InvalidCommissionRate
			);

			// check and reserve `deposit`
			ensure!(
				deposit >= T::ExtraConfig::get_min_order_deposit(),
				Error::<T>::SubmitWithInvalidDeposit
			);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			// check deadline
			ensure!(
				frame_system::Pallet::<T>::block_number() < deadline,
				Error::<T>::SubmitWithInvalidDeadline
			);

			// check hammer price
			if hammer_price > Zero::zero() {
				ensure!(hammer_price > init_price, Error::<T>::InvalidHammerPrice);
			}

			let mut auction: BritishAuctionOf<T> = BritishAuction {
				currency_id,
				hammer_price,
				min_raise,
				deposit,
				init_price,
				deadline,
				allow_delay,
				items: Vec::with_capacity(items.len()),
				commission_rate,
			};

			ensure_one_royalty!(items);
			reserve_and_push_tokens::<_, _, _, T::VFE>(Some(&who), &items, &mut auction.items)?;

			// generate an auction id
			let auction_id = T::ExtraConfig::get_then_inc_id()?;

			// save auction information.
			BritishAuctions::<T>::insert(&who, auction_id, auction);

			let auction_bid: BritishAuctionBidOf<T> = BritishAuctionBid {
				last_bid_price: init_price,
				last_bid_account: None,
				last_bid_block: Zero::zero(),
				commission_agent: None,
				commission_data: None,
			};

			BritishAuctionBids::<T>::insert(auction_id, auction_bid);

			// emit event.
			Self::deposit_event(Event::CreatedBritishAuction(who, auction_id));
			Ok(().into())
		}

		/// Bid
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn bid_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] price: Balance,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
			commission_agent: Option<T::AccountId>,
			commission_data: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let purchaser = ensure_signed(origin)?;
			let auction_owner = T::Lookup::lookup(auction_owner)?;
			ensure!(purchaser != auction_owner, Error::<T>::SelfBid);
			if let Some(c) = &commission_agent {
				ensure!(&purchaser != c, Error::<T>::SenderTakeCommission);
			}

			let auction: BritishAuctionOf<T> = Self::british_auctions(&auction_owner, auction_id)
				.ok_or(Error::<T>::BritishAuctionNotFound)?;
			let auction_bid: BritishAuctionBidOf<T> = Self::british_auction_bids(auction_id)
				.ok_or(Error::<T>::BritishAuctionBidNotFound)?;

			// check deadline
			ensure!(
				get_deadline::<T>(
					auction.allow_delay,
					auction.deadline,
					auction_bid.last_bid_block
				) >= frame_system::Pallet::<T>::block_number(),
				Error::<T>::BritishAuctionClosed,
			);

			// check hammer price
			if !auction.hammer_price.is_zero() && price >= auction.hammer_price {
				// delete the auction and release all assets reserved by this auction.
				Self::delete_british_auction(&auction_owner, auction_id)?;

				let (items, commission_agent) = to_item_vec!(auction, commission_agent);
				let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
				swap_assets::<T::MultiCurrency, T::VFE, _, _, _, _>(
					&purchaser,
					&auction_owner,
					auction.currency_id,
					auction.hammer_price,
					&items,
					&Self::treasury_account_id(),
					T::ExtraConfig::get_platform_fee_rate(),
					&beneficiary,
					royalty_rate,
					&commission_agent,
				)?;

				Self::deposit_event(Event::HammerBritishAuction(
					purchaser,
					auction_id,
					commission_agent,
					commission_data,
				));
				Ok(().into())
			} else {
				if auction_bid.last_bid_account.is_none() {
					ensure!(price >= auction.init_price, Error::<T>::PriceTooLow);
				}

				Self::save_british_bid(
					auction_bid,
					auction,
					price,
					purchaser.clone(),
					auction_id,
					commission_agent,
					commission_data,
				)?;

				Self::deposit_event(Event::BidBritishAuction(purchaser, auction_id));
				Ok(().into())
			}
		}

		/// redeem
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn redeem_british_auction(
			origin: OriginFor<T>,
			auction_owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let auction_owner = T::Lookup::lookup(auction_owner)?;
			let (auction, auction_bid) = Self::delete_british_auction(&auction_owner, auction_id)?;
			ensure!(
				get_deadline::<T>(
					auction.allow_delay,
					auction.deadline,
					auction_bid.last_bid_block
				) < frame_system::Pallet::<T>::block_number(),
				Error::<T>::CannotRedeemAuctionUntilDeadline
			);
			ensure!(auction_bid.last_bid_account.is_some(), Error::<T>::CannotRedeemAuctionNoBid);
			let purchaser = auction_bid.last_bid_account.expect("Must be Some");

			let commission_agent = auction_bid.commission_agent.clone();
			let (items, commission_agent) = to_item_vec!(auction, commission_agent);
			let (beneficiary, royalty_rate) = ensure_one_royalty!(items);
			swap_assets::<T::MultiCurrency, T::VFE, _, _, _, _>(
				&purchaser,
				&auction_owner,
				auction.currency_id,
				auction_bid.last_bid_price,
				&items,
				&Self::treasury_account_id(),
				T::ExtraConfig::get_platform_fee_rate(),
				&beneficiary,
				royalty_rate,
				&commission_agent,
			)?;

			Self::deposit_event(Event::RedeemedBritishAuction(
				purchaser,
				auction_id,
				commission_agent,
				auction_bid.commission_data,
			));
			Ok(().into())
		}

		/// remove an auction by auction owner.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_british_auction(
			origin: OriginFor<T>,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let (_, bid) = Self::delete_british_auction(&who, auction_id)?;
			ensure!(bid.last_bid_account.is_none(), Error::<T>::CannotRemoveAuction);
			Self::deposit_event(Event::RemovedBritishAuction(who, auction_id));
			Ok(().into())
		}

		/// remove an expired british auction by auction owner.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_expired_british_auction(
			origin: OriginFor<T>,
			auction_owner: T::AccountId,
			#[pallet::compact] auction_id: GlobalId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let auction: BritishAuctionOf<T> = Self::british_auctions(&auction_owner, auction_id)
				.ok_or(Error::<T>::BritishAuctionNotFound)?;
			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(current_block >= auction.deadline, Error::<T>::CannotRemoveAuction);
			let (_, bid) = Self::delete_british_auction(&auction_owner, auction_id)?;
			ensure!(bid.last_bid_account.is_none(), Error::<T>::CannotRemoveAuction);
			Self::deposit_event(Event::RemovedBritishAuction(who, auction_id));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn treasury_account_id() -> T::AccountId {
		sp_runtime::traits::AccountIdConversion::<T::AccountId>::into_account(
			&T::TreasuryPalletId::get(),
		)
	}

	fn delete_british_auction(
		who: &T::AccountId,
		auction_id: GlobalId,
	) -> Result<(BritishAuctionOf<T>, BritishAuctionBidOf<T>), DispatchError> {
		delete_auction!(
			BritishAuctionBids,
			BritishAuctions,
			who,
			auction_id,
			BritishAuctionBidNotFound,
			BritishAuctionNotFound,
		)
	}

	fn delete_dutch_auction(
		who: &T::AccountId,
		auction_id: GlobalId,
	) -> Result<(DutchAuctionOf<T>, DutchAuctionBidOf<T>), DispatchError> {
		delete_auction!(
			DutchAuctionBids,
			DutchAuctions,
			who,
			auction_id,
			DutchAuctionBidNotFound,
			DutchAuctionNotFound,
		)
	}

	fn save_dutch_bid(
		auction_bid: DutchAuctionBidOf<T>,
		auction: DutchAuctionOf<T>,
		price: Balance,
		purchaser: T::AccountId,
		auction_id: GlobalId,
		commission_agent: Option<T::AccountId>,
		commission_data: Option<Vec<u8>>,
	) -> DispatchResult {
		save_bid!(
			auction_bid,
			auction,
			price,
			purchaser,
			auction_id,
			DutchAuctionBids,
			commission_agent,
			commission_data,
		);
		Ok(())
	}

	fn save_british_bid(
		auction_bid: BritishAuctionBidOf<T>,
		auction: BritishAuctionOf<T>,
		price: Balance,
		purchaser: T::AccountId,
		auction_id: GlobalId,
		commission_agent: Option<T::AccountId>,
		commission_data: Option<Vec<u8>>,
	) -> DispatchResult {
		save_bid!(
			auction_bid,
			auction,
			price,
			purchaser,
			auction_id,
			BritishAuctionBids,
			commission_agent,
			commission_data,
		);
		Ok(())
	}
}
