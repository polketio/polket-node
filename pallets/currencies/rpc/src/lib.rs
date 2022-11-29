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

pub use pallet_currencies_rpc_runtime_api::CurrenciesApi as CurrenciesRuntimeApi;

#[rpc(client, server)]
pub trait CurrenciesApi<BlockHash, AccountId, AssetId, Balance> {
	#[method(name = "currencies_totalIssuance")]
	fn total_issuance(&self, asset: AssetId, at: Option<BlockHash>) -> RpcResult<Balance>;

	#[method(name = "currencies_minimumBalance")]
	fn minimum_balance(&self, asset: AssetId, at: Option<BlockHash>) -> RpcResult<Balance>;

	#[method(name = "currencies_balance")]
	fn balance(&self, asset: AssetId, who: AccountId, at: Option<BlockHash>)
		-> RpcResult<Balance>;

	#[method(name = "currencies_reducibleBalance")]
	fn reducible_balance(
		&self,
		asset: AssetId,
		who: AccountId,
		keep_alive: bool,
		at: Option<BlockHash>,
	) -> RpcResult<Balance>;
}

/// Provides RPC methods to query currencies detail.
pub struct Currencies<C, B> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Currencies<C, B> {
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
impl<C, Block, AccountId, AssetId, Balance>
	CurrenciesApiServer<<Block as BlockT>::Hash, AccountId, AssetId, Balance> for Currencies<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: CurrenciesRuntimeApi<Block, AccountId, AssetId, Balance>,
	AccountId: Codec,
	AssetId: Codec,
	Balance: Codec,
{
	fn total_issuance(&self, asset: AssetId, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.total_issuance(&at, asset).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}

	fn minimum_balance(&self, asset: AssetId, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.minimum_balance(&at, asset).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}

	fn balance(
		&self,
		asset: AssetId,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.balance(&at, asset, who).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}

	fn reducible_balance(
		&self,
		asset: AssetId,
		who: AccountId,
		keep_alive: bool,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.reducible_balance(&at, asset, who, keep_alive).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to get value.",
				Some(e.to_string()),
			))
			.into()
		})
	}
}
