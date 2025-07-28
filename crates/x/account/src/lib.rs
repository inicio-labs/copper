pub mod genesis;
pub mod keeper;
pub mod types;

use core::marker::PhantomData;

use copper_base::{
	Address, GenesisInitializer, Initialized, MsgHandler, SignerExtractor, Uninitialized,
	context::MutContext, msg::RoutableMsg,
};
use copper_store::{GetKVStore, InsertKVStore};
use genesis::AccountGenesis;
use nebz::NonEmptyBz;

use self::keeper::AccountKeeper;

pub struct AccountFacet<G> {
	keeper: AccountKeeper,
	_genesis: PhantomData<G>,
}

impl<G> AccountFacet<G> {
	pub fn keeper(&self) -> &AccountKeeper {
		&self.keeper
	}
}

impl<G> AccountFacet<G> {
	pub const NAME: &str = "account";

	const PREFIX: NonEmptyBz<&[u8]> = NonEmptyBz::from_borrowed_array(b"account_facet:").as_slice();

	pub fn new() -> Self {
		Self { keeper: AccountKeeper::new(Self::PREFIX), _genesis: PhantomData }
	}
}

impl<G> Default for AccountFacet<G> {
	fn default() -> Self {
		Self::new()
	}
}

impl<S> GenesisInitializer<S, &AccountGenesis> for AccountFacet<Uninitialized>
where
	S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>>,
	NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
{
	type Initialized = AccountFacet<Initialized>;

	fn init_genesis(
		self,
		store: &mut S,
		genesis: &AccountGenesis,
	) -> anyhow::Result<Self::Initialized> {
		self.keeper.init_accounts(store, genesis.addresses())?;

		Ok(AccountFacet { keeper: self.keeper, _genesis: PhantomData })
	}
}

impl<S> MsgHandler<S> for AccountFacet<Initialized>
where
	S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>>,
	NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
{
	fn handle_msg<'t, 's>(
		&self,
		_ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<()> {
		anyhow::bail!("unknown msg type: {}", msg.msg_id());
	}
}

impl<S> SignerExtractor<S> for AccountFacet<Initialized> {
	fn extract_signers<'t, 's>(
		&self,
		_ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<Vec<Address>> {
		anyhow::bail!("unknown msg type: {}", msg.msg_id());
	}
}
