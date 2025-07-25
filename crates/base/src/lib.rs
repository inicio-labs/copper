pub mod block;
pub mod msg;
pub mod tx;

use borsh::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

use self::{block::BlockHeight, msg::RoutableMsg};

pub type Address = [u8; 20];

pub fn derive_address(vk: &VerifyingKey) -> Address {
	let hash = Sha256::digest(vk.as_bytes());
	let mut address = [0; 20];
	address.copy_from_slice(&hash[hash.len() - 20..]);
	address
}

pub trait Module {
	type Store;

	fn name(&self) -> &'static str;

	fn handle_msg<'a>(
		&self,
		ctx: &'a mut MutContext<'a, Self::Store>,
		msg: &RoutableMsg,
	) -> anyhow::Result<()>;

	fn extract_signers(&self, msg: &RoutableMsg) -> anyhow::Result<Vec<Address>>;
}

pub struct MutContext<'a, S> {
	store: &'a mut S,
	height: BlockHeight,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct Coin {
	denom: String,
	amount: u128,
}

impl<S> MutContext<'_, S> {
	pub fn height(&self) -> BlockHeight {
		self.height
	}

	pub fn store(&mut self) -> &mut S {
		self.store
	}
}

impl Coin {
	pub fn new(denom: String, amount: u128) -> Self {
		Self { denom, amount }
	}

	pub fn denom(&self) -> &str {
		&self.denom
	}

	pub fn amount(&self) -> u128 {
		self.amount
	}
}
