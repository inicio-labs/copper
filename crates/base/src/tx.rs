use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;
use sha2::{Digest, Sha256};

use crate::msg::RoutableMsg;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Tx {
	msgs: Vec<RoutableMsg>,
	signature: Bytes,
	pub_key: Bytes,
	sequence: u64,
}

#[derive(Debug, Clone, BorshSerialize)]
struct SignDoc<'a> {
	chain_id: &'a [u8],
	msgs: &'a [RoutableMsg],
	sequence: u64,
}

impl Tx {
	pub fn msgs(&self) -> &[RoutableMsg] {
		&self.msgs
	}

	pub fn signature(&self) -> &Bytes {
		&self.signature
	}

	pub fn pub_key(&self) -> &Bytes {
		&self.pub_key
	}

	pub fn sequence(&self) -> u64 {
		self.sequence
	}

	pub fn hash_to_sign(&self, chain_id: &[u8]) -> [u8; 32] {
		let sign_doc = SignDoc { chain_id, msgs: &self.msgs, sequence: self.sequence };

		let mut hasher = Sha256::new();
		sign_doc.serialize(&mut hasher).unwrap();
		hasher.finalize().into()
	}

	pub fn dissolve(self) -> (Vec<RoutableMsg>, Bytes, Bytes, u64) {
		let Self { msgs, signature, pub_key, sequence } = self;
		(msgs, signature, pub_key, sequence)
	}
}
