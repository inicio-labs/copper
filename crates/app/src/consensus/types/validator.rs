use core::{
	fmt::{self, Display},
	slice,
};

use std::sync::Arc;

use borsh::{BorshDeserialize, BorshSerialize};
use malachitebft_core_types::{Context, Validator, ValidatorSet, VotingPower};
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::{PublicKey, proto};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::consensus::context::ConsensusContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopperValidatorSet {
	pub validators: Arc<Vec<CopperValidator>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopperValidator {
	pub address: ValidatorAddress,
	pub public_key: PublicKey,
	pub voting_power: VotingPower,
}

#[derive(
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	BorshSerialize,
	BorshDeserialize,
	Serialize,
	Deserialize,
)]
pub struct ValidatorAddress([u8; Self::LENGTH]);

impl CopperValidatorSet {
	pub fn new(mut validators: Vec<CopperValidator>) -> Option<Self> {
		validators.dedup();

		if validators.is_empty() {
			return None;
		}

		Some(Self { validators: Arc::new(validators) })
	}

	pub fn len(&self) -> usize {
		self.validators.len()
	}

	pub fn is_empty(&self) -> bool {
		self.validators.is_empty()
	}

	pub fn iter(&self) -> slice::Iter<'_, CopperValidator> {
		self.validators.iter()
	}

	pub fn total_voting_power(&self) -> VotingPower {
		self.validators.iter().map(|v| v.voting_power).sum()
	}

	pub fn get_by_index(&self, index: usize) -> Option<&CopperValidator> {
		self.validators.get(index)
	}

	pub fn get_by_address(&self, address: &ValidatorAddress) -> Option<&CopperValidator> {
		self.validators.iter().find(|v| &v.address == address)
	}

	pub fn get_by_public_key(&self, public_key: &PublicKey) -> Option<&CopperValidator> {
		self.validators.iter().find(|v| &v.public_key == public_key)
	}

	pub fn get_pub_keys(&self) -> Vec<PublicKey> {
		self.validators.iter().map(|v| v.public_key).collect()
	}
}

impl CopperValidator {
	pub fn new(public_key: PublicKey, voting_power: VotingPower) -> Self {
		Self { address: ValidatorAddress::from_public_key(&public_key), public_key, voting_power }
	}
}

impl ValidatorAddress {
	const LENGTH: usize = 20;

	pub const fn new(value: [u8; Self::LENGTH]) -> Self {
		Self(value)
	}

	pub fn from_public_key(public_key: &PublicKey) -> Self {
		let hash: [u8; 32] = Sha256::digest(public_key.as_bytes()).into();

		let mut address = [0; Self::LENGTH];
		address.copy_from_slice(&hash[..Self::LENGTH]);

		Self(address)
	}

	pub fn get(&self) -> &[u8; Self::LENGTH] {
		&self.0
	}

	pub fn into_inner(self) -> [u8; Self::LENGTH] {
		self.0
	}
}

impl Display for ValidatorAddress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for byte in self.0.iter() {
			write!(f, "{:02X}", byte)?;
		}

		Ok(())
	}
}

impl fmt::Debug for ValidatorAddress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Address({})", self)
	}
}

impl ValidatorSet<ConsensusContext> for CopperValidatorSet {
	fn count(&self) -> usize {
		self.validators.len()
	}

	fn total_voting_power(&self) -> VotingPower {
		self.validators.iter().map(|v| v.voting_power).sum()
	}

	fn get_by_address(
		&self,
		address: &<ConsensusContext as Context>::Address,
	) -> Option<&<ConsensusContext as Context>::Validator> {
		self.validators.iter().find(|v| &v.address == address)
	}

	fn get_by_index(&self, index: usize) -> Option<&<ConsensusContext as Context>::Validator> {
		self.validators.get(index)
	}
}

impl Validator<ConsensusContext> for CopperValidator {
	fn address(&self) -> &<ConsensusContext as Context>::Address {
		&self.address
	}

	fn public_key(&self) -> &PublicKey {
		&self.public_key
	}

	fn voting_power(&self) -> VotingPower {
		self.voting_power
	}
}

impl malachitebft_core_types::Address for ValidatorAddress {}

impl Protobuf for ValidatorAddress {
	type Proto = proto::Address;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		proto
			.value
			.as_ref()
			.try_into()
			.map(Self)
			.map_err(|_| {
				format!(
					"invalid address length {}: expected {}",
					proto.value.len(),
					Self::LENGTH,
				)
			})
			.map_err(Error::Other)
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		Ok(proto::Address { value: self.0.to_vec().into() })
	}
}
