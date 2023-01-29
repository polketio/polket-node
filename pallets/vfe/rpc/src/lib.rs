use std::sync::Arc;

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_vfe_rpc_runtime_api::VfeApi as VFERuntimeApi;

#[rpc(client, server)]
pub trait VfeApi<BlockHash, AccountId, BrandId, ItemId, VFEDetail, Balance> {
	#[method(name = "vfe_getVFEDetailsByAddress")]
	fn get_vfe_details_by_address(
		&self,
		account: AccountId,
		brand_id: BrandId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<VFEDetail>>;

	#[method(name = "vfe_getChargingCosts")]
	fn get_charging_costs(
		&self,
		brand_id: BrandId,
		item: ItemId,
		charge_num: u16,
		at: Option<BlockHash>,
	) -> RpcResult<Balance>;

	#[method(name = "vfe_getLevelUpCosts")]
	fn get_level_up_costs(
		&self,
		who: AccountId,
		brand_id: BrandId,
		item: ItemId,
		at: Option<BlockHash>,
	) -> RpcResult<Balance>;
}

/// Provides RPC methods to query vfe detail.
pub struct Vfe<C, B> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Vfe<C, B> {
	/// Creates a new instance of the `VFE` helper.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

pub enum Error {
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

#[async_trait]
impl<C, Block, AccountId, BrandId, ItemId, VFEDetail, Balance>
	VfeApiServer<<Block as BlockT>::Hash, AccountId, BrandId, ItemId, VFEDetail, Balance>
	for Vfe<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: VFERuntimeApi<Block, AccountId, BrandId, ItemId, VFEDetail, Balance>,
	AccountId: Codec,
	BrandId: Codec,
	ItemId: Codec,
	VFEDetail: Codec,
	Balance: Codec,
{
	fn get_vfe_details_by_address(
		&self,
		account: AccountId,
		brand_id: BrandId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<VFEDetail>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_vfe_details_by_address(&at, account, brand_id).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}

	fn get_charging_costs(
		&self,
		brand_id: BrandId,
		item: ItemId,
		charge_num: u16,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_charging_costs(&at, brand_id, item, charge_num).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}

	fn get_level_up_costs(
		&self,
		who: AccountId,
		brand_id: BrandId,
		item: ItemId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_level_up_costs(&at, who, brand_id, item).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}
}
