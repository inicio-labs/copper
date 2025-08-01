use bytes::Bytes;
use malachitebft_core_types::{Context, NilOrVal, Round, SignedExtension, Value, Vote, VoteType};
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::proto;

use crate::consensus::context::ConsensusContext;

use super::{ConsensusHeight, CopperValueId, ValidatorAddress};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CopperVote {
	pub typ: VoteType,
	pub height: ConsensusHeight,
	pub round: Round,
	pub value: NilOrVal<CopperValueId>,
	pub validator_address: ValidatorAddress,
	pub extension: Option<SignedExtension<ConsensusContext>>,
}

impl CopperVote {
	pub fn new_prevote(
		height: ConsensusHeight,
		round: Round,
		value: NilOrVal<CopperValueId>,
		validator_address: ValidatorAddress,
	) -> Self {
		Self { typ: VoteType::Prevote, height, round, value, validator_address, extension: None }
	}

	pub fn new_precommit(
		height: ConsensusHeight,
		round: Round,
		value: NilOrVal<CopperValueId>,
		address: ValidatorAddress,
	) -> Self {
		Self {
			typ: VoteType::Precommit,
			height,
			round,
			value,
			validator_address: address,
			extension: None,
		}
	}

	pub fn to_sign_bytes(&self) -> Bytes {
		let vote = Self { extension: None, ..self.clone() };

		Protobuf::to_bytes(&vote).unwrap()
	}

	pub fn from_sign_bytes(bytes: &[u8]) -> Result<Self, Error> {
		Protobuf::from_bytes(bytes)
	}
}

impl Vote<ConsensusContext> for CopperVote {
	fn height(&self) -> <ConsensusContext as Context>::Height {
		self.height
	}

	fn round(&self) -> Round {
		self.round
	}

	fn value(&self) -> &NilOrVal<<<ConsensusContext as Context>::Value as Value>::Id> {
		&self.value
	}

	fn take_value(self) -> NilOrVal<<<ConsensusContext as Context>::Value as Value>::Id> {
		self.value
	}

	fn vote_type(&self) -> VoteType {
		self.typ
	}

	fn validator_address(&self) -> &<ConsensusContext as Context>::Address {
		&self.validator_address
	}

	fn extension(&self) -> Option<&SignedExtension<ConsensusContext>> {
		self.extension.as_ref()
	}

	fn take_extension(&mut self) -> Option<SignedExtension<ConsensusContext>> {
		self.extension.take()
	}

	fn extend(self, extension: SignedExtension<ConsensusContext>) -> Self {
		Self { extension: Some(extension), ..self }
	}
}

impl Protobuf for CopperVote {
	type Proto = proto::Vote;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		Ok(Self {
			typ: malachitebft_test::decode_votetype(proto.vote_type()),
			height: ConsensusHeight::from_proto(proto.height)?,
			round: Round::new(proto.round),
			value: match proto.value {
				Some(value) => NilOrVal::Val(CopperValueId::from_proto(value)?),
				None => NilOrVal::Nil,
			},
			validator_address: proto
				.validator_address
				.ok_or_else(|| Error::missing_field::<Self::Proto>("validator_address"))
				.and_then(ValidatorAddress::from_proto)?,
			extension: Default::default(),
		})
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		Ok(Self::Proto {
			vote_type: malachitebft_test::encode_votetype(self.typ).into(),
			height: self.height.to_proto()?,
			round: self.round.as_u32().expect("round should not be nil"),
			value: match &self.value {
				NilOrVal::Nil => None,
				NilOrVal::Val(v) => Some(v.to_proto()?),
			},
			validator_address: Some(self.validator_address.to_proto()?),
		})
	}
}
