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
pub trait VfeApi<BlockHash, AccountId, BrandId, VFEDetail> {
	#[method(name = "vfe_getVFEDetailsByAddress")]
	fn get_vfe_details_by_address(
		&self,
		account: AccountId,
		brand_id: BrandId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<VFEDetail>>;
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
impl<C, Block, AccountId, BrandId, VFEDetail>
	VfeApiServer<<Block as BlockT>::Hash, AccountId, BrandId, VFEDetail> for Vfe<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: VFERuntimeApi<Block, AccountId, BrandId, VFEDetail>,
	AccountId: Codec,
	BrandId: Codec,
	VFEDetail: Codec,
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
}
