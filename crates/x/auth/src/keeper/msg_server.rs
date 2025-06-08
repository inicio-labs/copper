use copper_collections::{CollectionError, context::Context, store::KVStore};
use copper_proto::cosmos::auth::v1beta1::MsgUpdateParams;
use prost::Message;

use crate::{
	keeper::{error::Error, genesis, keeper::AccountKeeper},
	types::{base_account::BaseAccount, genesis::GenesisState, params::Params},
};

impl<C: Context + Clone + 'static, KV: KVStore<C, CollectionError> + Clone + 'static>
	AccountKeeper<C, KV>
{
	pub fn update_params(&self, ctx: &C, msg: MsgUpdateParams) -> super::error::Result<()> {
		// Check authority
		if self.authority != msg.authority {
			return Err(Error::InvalidAuthority);
		}

		Params::validate(msg.params.unwrap()).map_err(|e| Error::InvalidParams(e))?;

		match msg.params {
			Some(params) => {
				self.params.set(ctx, &params.encode_to_vec())?;
			},
			None => {
				return Err(Error::FieldMissing);
			},
		}

		Ok(())
	}
}
