use core::fmt::{self, Display};

use borsh::{BorshDeserialize, BorshSerialize};
use malachitebft_proto::{Error, Protobuf};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSerialize, BorshDeserialize)]
pub struct ConsensusHeight(u64);

impl ConsensusHeight {
	pub const fn new(height: u64) -> Self {
		Self(height)
	}

	pub const fn as_u64(&self) -> u64 {
		self.0
	}

	pub fn increment(&self) -> Self {
		Self(self.0 + 1)
	}

	pub fn decrement(&self) -> Option<Self> {
		self.0.checked_sub(1).map(Self)
	}
}

impl Default for ConsensusHeight {
	fn default() -> Self {
		malachitebft_core_types::Height::ZERO
	}
}

impl Display for ConsensusHeight {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl fmt::Debug for ConsensusHeight {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Height({})", self.0)
	}
}

impl malachitebft_core_types::Height for ConsensusHeight {
	const ZERO: Self = Self(0);
	const INITIAL: Self = Self(1);

	fn increment_by(&self, n: u64) -> Self {
		Self(self.0 + n)
	}

	fn decrement_by(&self, n: u64) -> Option<Self> {
		Some(Self(self.0.saturating_sub(n)))
	}

	fn as_u64(&self) -> u64 {
		self.0
	}
}

impl Protobuf for ConsensusHeight {
	type Proto = u64;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		Ok(Self(proto))
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		Ok(self.0)
	}
}
