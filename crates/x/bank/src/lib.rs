pub mod genesis;
pub mod keeper;
pub mod types;

use core::marker::PhantomData;

use copper_base::{
	Address, GenesisInitializer, Initialized, MsgHandler, SignerExtractor, Uninitialized,
	context::MutContext, msg::RoutableMsg,
};
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use genesis::BankGenesis;
use nebz::NonEmptyBz;
use types::CoinSend;

use self::keeper::BankKeeper;

pub struct BankFacet<'a, G> {
	keeper: BankKeeper<'a>,
	_genesis: PhantomData<G>,
}

impl<'a, G> BankFacet<'a, G> {
	pub fn keeper(&self) -> &BankKeeper<'a> {
		&self.keeper
	}
}

impl<'a> BankFacet<'a, Uninitialized> {
	pub const NAME: &'static str = "bank";

	const PREFIX: NonEmptyBz<&'static [u8]> =
		NonEmptyBz::from_borrowed_array(b"bank_facet:").as_slice();

	pub fn new() -> Self {
		Self { keeper: BankKeeper::new(Self::PREFIX), _genesis: PhantomData }
	}
}

impl<'a> Default for BankFacet<'a, Uninitialized> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a, S> GenesisInitializer<S, &BankGenesis> for BankFacet<'a, Uninitialized>
where
	S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
	NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
{
	type Initialized = BankFacet<'a, Initialized>;

	fn init_genesis(
		self,
		store: &mut S,
		genesis: &BankGenesis,
	) -> anyhow::Result<Self::Initialized> {
		self.keeper.init_balances(store, genesis.balances())?;

		Ok(BankFacet { keeper: self.keeper, _genesis: PhantomData })
	}
}

impl<'a, S> SignerExtractor<S> for BankFacet<'a, Initialized> {
	fn extract_signers<'t, 's>(
		&self,
		_ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<Vec<Address>> {
		let signers = match msg.msg_id() {
			CoinSend::ID => {
				borsh::from_slice::<CoinSend>(msg.content()).map(|c| vec![*c.from()])?
			},
			_ => anyhow::bail!("unknown msg type: {}", msg.msg_id()),
		};

		Ok(signers)
	}
}

impl<'a, S> MsgHandler<S> for BankFacet<'a, Initialized>
where
	S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
	NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
{
	fn handle_msg<'t, 's>(
		&self,
		ctx: &mut MutContext<'t, 's, S>,
		msg: &RoutableMsg,
	) -> anyhow::Result<()> {
		match msg.msg_id() {
			CoinSend::ID => {
				let c: CoinSend = borsh::from_slice(msg.content())?;

				self.keeper.send_coin(ctx.store(), c.from(), c.to(), c.coin())?;

				Ok(())
			},
			_ => anyhow::bail!("unknown msg type: {}", msg.msg_id()),
		}
	}
}
