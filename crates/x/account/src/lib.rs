pub mod keeper;
pub mod types;

use copper_base::{Address, Module, MutContext, msg::RoutableMsg};
use copper_store::iavl::IavlStore;
use iavl::kvstore::redb::RedbStore;
use nebz::NonEmptyBz;

use self::keeper::AccountKeeper;

pub struct AccountModule {
	keeper: AccountKeeper,
}

impl AccountModule {
	const NAME: &str = "account";

	const PREFIX: NonEmptyBz<&[u8; 15]> = NonEmptyBz::from_borrowed_array(b"account_module:");

	pub fn new() -> Self {
		Self { keeper: AccountKeeper::new(Self::PREFIX.as_ref_slice()) }
	}

	pub fn keeper(&self) -> &AccountKeeper {
		&self.keeper
	}
}

impl Default for AccountModule {
	fn default() -> Self {
		Self::new()
	}
}

impl Module for AccountModule {
	type Store = IavlStore<RedbStore>;

	fn name(&self) -> &'static str {
		Self::NAME
	}

	fn handle_msg(
		&self,
		_ctx: &mut MutContext<'_, Self::Store>,
		_msg: &RoutableMsg,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn extract_signers(&self, _msg: &RoutableMsg) -> anyhow::Result<Vec<Address>> {
		Ok(vec![])
	}
}
