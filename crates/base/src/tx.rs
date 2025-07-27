use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;
use sha2::{Digest, Sha256};

use crate::msg::RoutableMsg;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Tx {
	msgs: Vec<RoutableMsg>,
	signer_info_signature_pairs: Vec<(SignerInfo, Bytes)>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SignerInfo {
	pub_key: Bytes,
	sequence: u128,
}

#[derive(Debug, Clone, BorshSerialize)]
struct SignDoc<'a> {
	chain_id: &'a [u8],
	msgs: &'a [RoutableMsg],
	signer_info_signature_pairs: &'a [(SignerInfo, Bytes)],
}

impl Tx {
	pub fn msgs(&self) -> &[RoutableMsg] {
		&self.msgs
	}

	pub fn signer_info_signature_pairs(&self) -> &[(SignerInfo, Bytes)] {
		&self.signer_info_signature_pairs
	}

	pub fn hash_to_sign(&self, chain_id: &[u8]) -> [u8; 32] {
		let sign_doc = SignDoc {
			chain_id,
			msgs: self.msgs(),
			signer_info_signature_pairs: self.signer_info_signature_pairs(),
		};

		let mut hasher = Sha256::new();

		// unwrap is safe here because write to hasher is infalliable.
		sign_doc.serialize(&mut hasher).unwrap();

		hasher.finalize().into()
	}

	pub fn dissolve(self) -> (Vec<RoutableMsg>, Vec<(SignerInfo, Bytes)>) {
		let Self { msgs, signer_info_signature_pairs } = self;
		(msgs, signer_info_signature_pairs)
	}
}
