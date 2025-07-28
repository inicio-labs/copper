use copper_base::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGenesis {
	pub accounts: Vec<Address>,
}

impl AccountGenesis {
	pub fn new(accounts: Vec<Address>) -> Self {
		Self { accounts }
	}

	pub fn addresses(&self) -> &[Address] {
		&self.accounts
	}
}
