use borsh::{BorshDeserialize, BorshSerialize};
use copper_base::{Address, Coin, msg::Msg};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MsgSend {
	from: Address,
	to: Address,
	coin: Coin,
}

impl MsgSend {
	pub(crate) const ROUTE: &[u8] = concat!("bank/", "send").as_bytes();

	pub fn dissolve(self) -> (Address, Address, Coin) {
		let Self { from, to, coin } = self;
		(from, to, coin)
	}
}

impl Msg for MsgSend {
	fn signers(&self) -> Vec<Address> {
		vec![self.from]
	}
}
