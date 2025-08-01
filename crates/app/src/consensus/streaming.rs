mod proposal_proto;

mod heap;

use std::collections::{BTreeMap, HashSet};

use bytes::Bytes;
use malachitebft_app_channel::app::{
	streaming::{Sequence, StreamId, StreamMessage},
	types::{
		PeerId,
		core::{Context, Proposal, ProposalPart as CoreProposalPart, Round},
	},
};
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::{ProposalFin, codec, proto};
use proposal_proto::proposal_part::Part;

use super::{
	context::ConsensusContext,
	types::{ConsensusHeight, CopperValue, ValidatorAddress},
};

use self::heap::MinHeap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperProposalParts {
	pub height: ConsensusHeight,
	pub round: Round,
	pub proposer: ValidatorAddress,
	pub parts: Vec<CopperProposalPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperProposal {
	pub height: ConsensusHeight,
	pub round: Round,
	pub value: CopperValue,
	pub pol_round: Round,
	pub validator_address: ValidatorAddress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopperProposalPart {
	Init(ProposalInit),
	Data(CopperValue),
	Fin(ProposalFin),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalInit {
	pub height: ConsensusHeight,
	pub round: Round,
	pub pol_round: Round,
	pub proposer: ValidatorAddress,
}

#[derive(Debug, Default)]
pub struct PartStreamMap {
	streams: BTreeMap<(PeerId, StreamId), StreamState>,
}

#[derive(Debug, Default)]
struct StreamState {
	buffer: MinHeap<CopperProposalPart>,
	init_info: Option<ProposalInit>,
	seen_sequences: HashSet<Sequence>,
	total_messages: usize,
	fin_received: bool,
}

impl CopperProposalParts {
	pub fn init(&self) -> Option<&ProposalInit> {
		self.parts.iter().find_map(CopperProposalPart::as_init)
	}

	pub fn fin(&self) -> Option<&ProposalFin> {
		self.parts.iter().find_map(CopperProposalPart::as_fin)
	}
}

impl CopperProposal {
	pub fn new(
		height: ConsensusHeight,
		round: Round,
		value: CopperValue,
		pol_round: Round,
		validator_address: ValidatorAddress,
	) -> Self {
		Self { height, round, value, pol_round, validator_address }
	}

	pub fn to_sign_bytes(&self) -> Bytes {
		Protobuf::to_bytes(self).unwrap()
	}

	pub fn from_sign_bytes(bytes: &[u8]) -> Result<Self, Error> {
		Protobuf::from_bytes(bytes)
	}
}

impl CopperProposalPart {
	pub fn get_type(&self) -> &'static str {
		match self {
			Self::Init(_) => "init",
			Self::Data(_) => "data",
			Self::Fin(_) => "fin",
		}
	}

	pub fn as_init(&self) -> Option<&ProposalInit> {
		match self {
			CopperProposalPart::Init(init) => Some(init),
			_ => None,
		}
	}

	pub fn as_data(&self) -> Option<&CopperValue> {
		match self {
			CopperProposalPart::Data(value) => Some(value),
			_ => None,
		}
	}

	pub fn as_fin(&self) -> Option<&ProposalFin> {
		match self {
			CopperProposalPart::Fin(fin) => Some(fin),
			_ => None,
		}
	}

	pub fn to_sign_bytes(&self) -> Bytes {
		Protobuf::to_bytes(self).unwrap()
	}
}

impl ProposalInit {
	pub fn new(
		height: ConsensusHeight,
		round: Round,
		pol_round: Round,
		proposer: ValidatorAddress,
	) -> Self {
		Self { height, round, pol_round, proposer }
	}
}

impl PartStreamMap {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(
		&mut self,
		peer_id: PeerId,
		msg: StreamMessage<CopperProposalPart>,
	) -> Option<CopperProposalParts> {
		let stream_id = msg.stream_id.clone();
		let state = self.streams.entry((peer_id, stream_id.clone())).or_default();

		if !state.seen_sequences.insert(msg.sequence) {
			// We have already seen a message with this sequence number.
			return None;
		}

		let result = state.insert(msg);

		if state.is_done() {
			self.streams.remove(&(peer_id, stream_id));
		}

		result
	}
}

impl StreamState {
	fn is_done(&self) -> bool {
		self.init_info.is_some() && self.fin_received && self.buffer.len() == self.total_messages
	}

	fn insert(&mut self, msg: StreamMessage<CopperProposalPart>) -> Option<CopperProposalParts> {
		if msg.is_first() {
			self.init_info = msg.content.as_data().and_then(CopperProposalPart::as_init).cloned();
		}

		if msg.is_fin() {
			self.fin_received = true;
			self.total_messages = msg.sequence as usize + 1;
		}

		self.buffer.push(msg);

		if self.is_done() {
			let ProposalInit { height, round, proposer, .. } = self.init_info.take()?;

			let height = ConsensusHeight::new(height.as_u64());

			Some(CopperProposalParts { height, round, proposer, parts: self.buffer.drain() })
		} else {
			None
		}
	}
}

impl Proposal<ConsensusContext> for CopperProposal {
	fn height(&self) -> <ConsensusContext as Context>::Height {
		self.height
	}

	fn round(&self) -> Round {
		self.round
	}

	fn value(&self) -> &<ConsensusContext as Context>::Value {
		&self.value
	}

	fn take_value(self) -> <ConsensusContext as Context>::Value {
		self.value
	}

	fn pol_round(&self) -> Round {
		self.pol_round
	}

	fn validator_address(&self) -> &<ConsensusContext as Context>::Address {
		&self.validator_address
	}
}

impl CoreProposalPart<ConsensusContext> for CopperProposalPart {
	fn is_first(&self) -> bool {
		matches!(self, Self::Init(_))
	}

	fn is_last(&self) -> bool {
		matches!(self, Self::Fin(_))
	}
}

impl Protobuf for CopperProposal {
	type Proto = proto::Proposal;

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		Ok(Self::Proto {
			height: self.height.to_proto()?,
			round: self.round.as_u32().expect("round should not be nil"),
			value: self.value.to_proto().map(Some)?,
			pol_round: self.pol_round.as_u32(),
			validator_address: self.validator_address.to_proto().map(Some)?,
		})
	}

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		Ok(Self {
			height: ConsensusHeight::from_proto(proto.height)?,
			round: Round::new(proto.round),
			value: proto
				.value
				.ok_or("value")
				.map_err(Error::missing_field::<Self::Proto>)
				.and_then(CopperValue::from_proto)?,
			pol_round: Round::from(proto.pol_round),
			validator_address: proto
				.validator_address
				.ok_or("validator_address")
				.map_err(Error::missing_field::<Self::Proto>)
				.and_then(ValidatorAddress::from_proto)?,
		})
	}
}

impl Protobuf for CopperProposalPart {
	type Proto = proposal_proto::ProposalPart;

	fn from_proto(proto: Self::Proto) -> Result<Self, Error> {
		match proto.part.ok_or("part").map_err(Error::missing_field::<Self::Proto>)? {
			Part::Init(init) => Ok(Self::Init(ProposalInit {
				height: ConsensusHeight::new(init.height),
				round: Round::new(init.round),
				pol_round: Round::from(init.pol_round),
				proposer: init
					.proposer
					.ok_or("proposer")
					.map_err(Error::missing_field::<Self::Proto>)
					.and_then(ValidatorAddress::from_proto)?,
			})),
			Part::Data(data) => CopperValue::from_proto(data).map(Self::Data),
			Part::Fin(fin) => fin
				.signature
				.ok_or("signature")
				.map_err(Error::missing_field::<Self::Proto>)
				.and_then(codec::proto::decode_signature)
				.map(ProposalFin::new)
				.map(Self::Fin),
		}
	}

	fn to_proto(&self) -> Result<Self::Proto, Error> {
		match self {
			Self::Init(init) => Ok(Self::Proto {
				part: Some(Part::Init(proto::ProposalInit {
					height: init.height.as_u64(),
					round: init
						.round
						.as_u32()
						.ok_or_else(|| "round must be non-nil".into())
						.map_err(Error::Other)?,
					pol_round: init.pol_round.as_u32(),
					proposer: init.proposer.to_proto().map(Some)?,
				})),
			}),
			Self::Data(data) => {
				data.to_proto().map(Part::Data).map(Some).map(|part| Self::Proto { part })
			},
			Self::Fin(fin) => Ok(Self::Proto {
				part: Some(Part::Fin(proto::ProposalFin {
					signature: Some(codec::proto::encode_signature(&fin.signature)),
				})),
			}),
		}
	}
}
