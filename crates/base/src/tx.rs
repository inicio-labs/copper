use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;
use sha2::{Digest, Sha256};

use crate::msg::RoutableMsg;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct Tx<S> {
	msgs: Vec<RoutableMsg>,
	signer_infos: Vec<SignerInfo>,
	sigs: S,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct SignerInfo {
	pub_key: Bytes,
	sequence: u128,
}

#[derive(Debug, Clone)]
pub struct Unsigned;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct Signed(Vec<Bytes>);

#[derive(Debug, Clone, BorshSerialize)]
struct SignDoc<'a> {
	chain_id: &'a [u8],
	msgs: &'a [RoutableMsg],
	signer_infos: &'a [SignerInfo],
}

impl<S> Tx<S> {
	pub fn msgs(&self) -> &[RoutableMsg] {
		&self.msgs
	}

	pub fn signer_infos(&self) -> &[SignerInfo] {
		&self.signer_infos
	}

	pub fn hash_to_sign(&self, chain_id: &[u8]) -> [u8; 32] {
		let sign_doc = SignDoc { chain_id, msgs: self.msgs(), signer_infos: self.signer_infos() };

		let mut hasher = Sha256::new();

		// unwrap is safe here because write to hasher is infallible.
		sign_doc.serialize(&mut hasher).unwrap();

		hasher.finalize().into()
	}
}

impl Tx<Unsigned> {
	pub fn new(msgs: Vec<RoutableMsg>, signer_infos: Vec<SignerInfo>) -> Self {
		Self { msgs, signer_infos, sigs: Unsigned }
	}

	pub fn into_signed(self, sigs: Vec<Bytes>) -> Result<Tx<Signed>, Self> {
		if self.signer_infos().len() != sigs.len() {
			return Err(self);
		}

		let Self { msgs, signer_infos, .. } = self;

		Ok(Tx { msgs, signer_infos, sigs: Signed(sigs) })
	}

	pub fn dissolve(self) -> (Vec<RoutableMsg>, Vec<SignerInfo>) {
		let Self { msgs, signer_infos, .. } = self;
		(msgs, signer_infos)
	}
}

impl Tx<Signed> {
	pub fn sigs(&self) -> &[Bytes] {
		&self.sigs.0
	}

	pub fn dissolve(self) -> (Vec<RoutableMsg>, Vec<SignerInfo>, Vec<Bytes>) {
		let Self { msgs, signer_infos, sigs } = self;
		(msgs, signer_infos, sigs.0)
	}
}

impl SignerInfo {
	pub fn new(pub_key: Bytes, sequence: u128) -> Self {
		Self { pub_key, sequence }
	}

	pub fn pub_key(&self) -> &Bytes {
		&self.pub_key
	}

	pub fn sequence(&self) -> u128 {
		self.sequence
	}
}
