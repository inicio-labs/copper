use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BaseAccount {
	pub_key: Option<Bytes>,
	sequence: u64,
}

impl BaseAccount {
	pub fn new(pub_key: impl Into<Option<Bytes>>, sequence: u64) -> Self {
		Self { pub_key: pub_key.into(), sequence }
	}

	pub fn pub_key(&self) -> Option<&Bytes> {
		self.pub_key.as_ref()
	}

	pub fn sequence(&self) -> u64 {
		self.sequence
	}

	pub fn dissolve(self) -> (Option<Bytes>, u64) {
		let Self { pub_key, sequence } = self;
		(pub_key, sequence)
	}
}
