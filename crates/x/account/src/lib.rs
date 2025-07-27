pub mod keeper;
pub mod types;

use nebz::NonEmptyBz;

use self::keeper::AccountKeeper;

pub struct AccountFacet {
	keeper: AccountKeeper,
}

impl AccountFacet {
	pub const NAME: &str = "account";

	const PREFIX: NonEmptyBz<&[u8]> =
		NonEmptyBz::from_borrowed_array(b"account_module:").as_slice();

	pub fn new() -> Self {
		Self { keeper: AccountKeeper::new(Self::PREFIX) }
	}

	pub fn keeper(&self) -> &AccountKeeper {
		&self.keeper
	}
}

impl Default for AccountFacet {
	fn default() -> Self {
		Self::new()
	}
}
