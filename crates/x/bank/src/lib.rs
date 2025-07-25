pub mod keeper;
pub mod types;

use borsh::BorshDeserialize;
use copper_base::{Address, Module, MutContext, msg::RoutableMsg};
use copper_store::iavl::IavlStore;
use iavl::kvstore::redb::RedbStore;
use nebz::NonEmptyBz;
use types::MsgSend;

use self::keeper::BankKeeper;

pub struct BankModule {
	keeper: BankKeeper,
}

impl BankModule {
	const NAME: &str = "bank";

	const PREFIX: NonEmptyBz<&[u8; 12]> = NonEmptyBz::from_borrowed_array(b"bank_module:");

	pub fn new() -> Self {
		Self { keeper: BankKeeper::new(Self::PREFIX.as_ref_slice()) }
	}
}

impl Default for BankModule {
	fn default() -> Self {
		Self::new()
	}
}

impl Module for BankModule {
	type Store = IavlStore<RedbStore>;

	fn name(&self) -> &'static str {
		Self::NAME
	}

	fn handle_msg<'a>(
		&self,
		ctx: &'a mut MutContext<'a, Self::Store>,
		msg: &RoutableMsg,
	) -> anyhow::Result<()> {
		match msg.route().as_ref() {
			MsgSend::ROUTE => {
				let (from, to, coin) = MsgSend::try_from_slice(msg.content())?.dissolve();

				self.keeper.send_coin(ctx.store(), &from, &to, coin)?;
			},
			_ => anyhow::bail!("unrecognized msg route"),
		}

		Ok(())
	}

	fn extract_signers(&self, msg: &RoutableMsg) -> anyhow::Result<Vec<Address>> {
		match msg.route().as_ref() {
			MsgSend::ROUTE => {
				let (from, _, _) = MsgSend::try_from_slice(msg.content())?.dissolve();
				Ok(vec![from])
			},
			_ => anyhow::bail!("unrecognized msg route"),
		}
	}
}
