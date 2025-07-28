use copper_base::{Address, coin::Coin};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankGenesis {
	balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::serde_as]
pub struct Balance {
	#[serde_as(as = "serde_with::Hex<serde_with::formats::Lowercase>")]
	address: Address,

	coins: Vec<Coin>,
}

impl BankGenesis {
	pub fn new(balances: Vec<Balance>) -> Self {
		Self { balances }
	}

	pub fn balances(&self) -> &[Balance] {
		&self.balances
	}

	pub fn dissolve(self) -> Vec<Balance> {
		self.balances
	}
}

impl Balance {
	pub fn new(address: Address, coins: Vec<Coin>) -> Self {
		Self { address, coins }
	}

	pub fn address(&self) -> &Address {
		&self.address
	}

	pub fn coins(&self) -> &[Coin] {
		&self.coins
	}

	pub fn dissolve(self) -> (Address, Vec<Coin>) {
		let Self { address, coins } = self;

		(address, coins)
	}
}
