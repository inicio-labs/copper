use core::fmt::{self, Display, Formatter};

use borsh::{BorshDeserialize, BorshSerialize};
use copper_base::block::{Block, BlockHash};
use malachitebft_core_types::Value;
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::proto;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CopperValue(Block);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub struct CopperValueId(BlockHash);

impl CopperValue {
	pub fn new(block: Block) -> Self {
		Self(block)
	}

	pub fn get(&self) -> &Block {
		&self.0
	}

	pub fn into_inner(self) -> Block {
		self.0
	}
}

impl CopperValueId {
	pub fn new(hash: BlockHash) -> Self {
		Self(hash)
	}

	pub fn get(&self) -> &BlockHash {
		&self.0
	}

	pub fn into_inner(self) -> BlockHash {
		self.0
	}
}

impl Display for CopperValueId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			const_hex::const_encode::<32, false>(&self.0).as_str(),
		)
	}
}

impl Value for CopperValue {
	type Id = CopperValueId;

	fn id(&self) -> Self::Id {
		let mut hasher = Sha256::new();
		self.0.serialize(&mut hasher).unwrap();
		CopperValueId(hasher.finalize().into())
	}
}

impl Protobuf for CopperValue {
	type Proto = proto::Value;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		proto
			.value
			.as_deref()
			.map(borsh::from_slice)
			.ok_or_else(|| Error::missing_field::<Self::Proto>("value"))?
			.map(Self::new)
			.map_err(|e| Error::Other(e.to_string()))
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		let bz = borsh::to_vec(self.get()).map_err(|e| Error::Other(e.to_string()))?;

		Ok(proto::Value { value: Some(bz.into()) })
	}
}

impl Protobuf for CopperValueId {
	type Proto = proto::ValueId;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		proto
			.value
			.as_deref()
			.ok_or("value")
			.map_err(Error::missing_field::<Self::Proto>)
			.map(TryFrom::try_from)?
			.map(Self::new)
			.map_err(|e| Error::Other(e.to_string()))
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		Ok(proto::ValueId { value: Some(self.get().to_vec().into()) })
	}
}
