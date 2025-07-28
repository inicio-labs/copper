use borsh::{BorshDeserialize, BorshSerialize};
use copper_base::{
	Address,
	coin::Coin,
	msg::{Msg, RoutableMsg},
};

use crate::BankFacet;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CoinSend {
	from: Address,
	to: Address,
	coin: Coin,
}

impl CoinSend {
	#[allow(dead_code)]
	pub(crate) const ID: &str = "coin-send";

	pub fn new(from: Address, to: Address, coin: Coin) -> Self {
		Self { from, to, coin }
	}

	pub fn from(&self) -> &Address {
		&self.from
	}

	pub fn to(&self) -> &Address {
		&self.to
	}

	pub fn coin(&self) -> &Coin {
		&self.coin
	}

	pub fn dissolve(self) -> (Address, Address, Coin) {
		let Self { from, to, coin } = self;
		(from, to, coin)
	}
}

impl Msg for CoinSend {
	fn to_routable_msg(&self) -> RoutableMsg {
		RoutableMsg::new(
			BankFacet::NAME.into(),
			Self::ID.into(),
			borsh::to_vec(&self).unwrap().into(),
		)
	}
}
