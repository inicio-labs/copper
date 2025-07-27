use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BaseAccount {
	pub_key: Option<Bytes>,
	sequence: u128,
}

impl BaseAccount {
	pub fn new<PK>(pub_key: PK, sequence: u128) -> Self
	where
		Option<Bytes>: From<PK>,
	{
		Self { pub_key: pub_key.into(), sequence }
	}

	pub fn pub_key(&self) -> Option<&Bytes> {
		self.pub_key.as_ref()
	}

	pub fn sequence(&self) -> u128 {
		self.sequence
	}

	pub fn dissolve(self) -> (Option<Bytes>, u128) {
		let Self { pub_key, sequence } = self;
		(pub_key, sequence)
	}
}
