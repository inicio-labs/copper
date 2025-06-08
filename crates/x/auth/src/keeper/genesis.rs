use copper_collections::{CollectionError, context::Context, store::KVStore};
use prost::Message;

use crate::{
	keeper::{genesis, keeper::AccountKeeper},
	types::{base_account::BaseAccount, genesis::GenesisState},
};

impl<C: Context + Clone + 'static, KV: KVStore<C, CollectionError> + Clone + 'static>
	AccountKeeper<C, KV>
{
	pub fn init_genesis(&self, ctx: &C, genesis_state: GenesisState) -> super::error::Result<()> {
		match genesis_state.inner.params {
			Some(params) => {
				self.params.set(ctx, &params.encode_to_vec())?;
			},
			None => {
				panic!("params are required");
			},
		}

		for account in genesis_state.inner.accounts {
			let account = BaseAccount { inner: account.to_msg()? };
			let addr = account.get_address();
			self.accounts.set(ctx, &addr.into(), &account.inner.encode_to_vec())?;
		}

		Ok(())
	}
}
