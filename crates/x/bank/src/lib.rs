pub mod keeper;
pub mod types;

use nebz::NonEmptyBz;

use self::keeper::BankKeeper;

pub struct BankFacet<'a> {
	keeper: BankKeeper<'a>,
}

impl<'a> BankFacet<'a> {
	pub const NAME: &'static str = "bank";

	const PREFIX: NonEmptyBz<&'static [u8]> =
		NonEmptyBz::from_borrowed_array(b"bank_facet:").as_slice();

	pub fn new() -> Self {
		Self { keeper: BankKeeper::new(Self::PREFIX) }
	}

	pub fn keeper(&self) -> &BankKeeper<'a> {
		&self.keeper
	}
}

impl Default for BankFacet<'_> {
	fn default() -> Self {
		Self::new()
	}
}
