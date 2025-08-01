use core::num::NonZeroU64;

use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

use crate::tx::{Signed, Tx};

pub type BlockHeight = NonZeroU64;
pub type BlockHash = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct Block {
	header: BlockHeader,
	txs: Vec<Tx<Signed>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct BlockHeader {
	height: BlockHeight,
	last_block_hash: Option<BlockHash>,
	app_hash: [u8; 32],
	txs_hash: [u8; 32],
}

impl Block {
	pub fn new(header: BlockHeader, txs: Vec<Tx<Signed>>) -> Self {
		Self { header, txs }
	}

	pub fn header(&self) -> &BlockHeader {
		&self.header
	}

	pub fn txs(&self) -> &[Tx<Signed>] {
		&self.txs
	}

	pub fn hash(&self) -> BlockHash {
		let mut hasher = Sha256::new();
		self.serialize(&mut hasher).unwrap();
		hasher.finalize().into()
	}

	pub fn dissolve(self) -> (BlockHeader, Vec<Tx<Signed>>) {
		let Self { header, txs } = self;
		(header, txs)
	}
}

impl BlockHeader {
	pub fn new(height: BlockHeight) -> Self {
		Self { height, last_block_hash: None, app_hash: [0; 32], txs_hash: [0; 32] }
	}

	pub fn height(&self) -> BlockHeight {
		self.height
	}

	pub fn last_block_hash(&self) -> Option<BlockHash> {
		self.last_block_hash
	}

	pub fn app_hash(&self) -> [u8; 32] {
		self.app_hash
	}

	pub fn txs_hash(&self) -> [u8; 32] {
		self.txs_hash
	}

	pub fn hash(&self) -> BlockHash {
		let mut hasher = Sha256::new();
		self.serialize(&mut hasher).unwrap();
		hasher.finalize().into()
	}

	pub fn dissolve(self) -> (BlockHeight, Option<BlockHash>, [u8; 32], [u8; 32]) {
		let Self { height, last_block_hash, app_hash, txs_hash } = self;
		(height, last_block_hash, app_hash, txs_hash)
	}
}
