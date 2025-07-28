pub mod block;
pub mod coin;
pub mod context;
pub mod msg;
pub mod tx;

use self::{context::MutContext, msg::RoutableMsg};

pub type Address = [u8; 20];

pub trait GenesisInitializer<S, G> {
	type Initialized;

	fn init_genesis(self, store: &mut S, genesis: G) -> anyhow::Result<Self::Initialized>;
}

pub trait MsgHandler<S> {
	fn handle_msg<'t, 's>(
		&self,
		ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<()>;
}

pub trait SignerExtractor<S> {
	fn extract_signers<'t, 's>(
		&self,
		ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<Vec<Address>>;
}

#[derive(Debug, Clone, Copy)]
pub struct Initialized;

#[derive(Debug, Clone, Copy)]
pub struct Uninitialized;
