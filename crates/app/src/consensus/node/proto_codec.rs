use bytes::Bytes;
use malachitebft_app_channel::app::{
	consensus::LivenessMsg,
	streaming::{StreamContent, StreamId, StreamMessage},
	types::{PeerId, ProposedValue, SignedConsensusMsg, codec::Codec, sync},
};
use malachitebft_core_types::{
	CommitCertificate, CommitSignature, NilOrVal, PolkaCertificate, PolkaSignature, Round,
	RoundCertificate, RoundCertificateType, RoundSignature, SignedProposal, SignedVote, Validity,
};
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::{Signature, codec, proto};
use prost::Message;

use crate::consensus::{
	context::ConsensusContext,
	streaming::{CopperProposal, CopperProposalPart},
	types::{ConsensusHeight, CopperValue, CopperValueId, CopperVote, ValidatorAddress},
};

#[derive(Copy, Clone, Debug)]
pub struct ProtobufCodec;

impl Codec<CopperValue> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<CopperValue, Self::Error> {
		Protobuf::from_bytes(&bytes)
	}

	fn encode(&self, msg: &CopperValue) -> Result<Bytes, Self::Error> {
		Protobuf::to_bytes(msg)
	}
}

impl Codec<CopperProposalPart> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<CopperProposalPart, Self::Error> {
		Protobuf::from_bytes(&bytes)
	}

	fn encode(&self, msg: &CopperProposalPart) -> Result<Bytes, Self::Error> {
		Protobuf::to_bytes(msg)
	}
}

impl Codec<Signature> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<Signature, Self::Error> {
		proto::Signature::decode(bytes.as_ref())
			.map_err(From::from)
			.and_then(codec::proto::decode_signature)
	}

	fn encode(&self, msg: &Signature) -> Result<Bytes, Self::Error> {
		Ok(proto::Signature { bytes: msg.to_bytes().to_vec().into() }.encode_to_vec().into())
	}
}

impl Codec<SignedConsensusMsg<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<SignedConsensusMsg<ConsensusContext>, Self::Error> {
		let proto = proto::SignedMessage::decode(bytes.as_ref())?;

		let signature = proto
			.signature
			.ok_or("signature")
			.map_err(Error::missing_field::<proto::SignedMessage>)
			.and_then(codec::proto::decode_signature)?;

		let proto_message =
			proto.message.ok_or("message").map_err(Error::missing_field::<proto::SignedMessage>)?;

		match proto_message {
			proto::signed_message::Message::Proposal(proto) => {
				let proposal = CopperProposal::from_proto(proto)?;
				Ok(SignedConsensusMsg::Proposal(SignedProposal::new(
					proposal, signature,
				)))
			},
			proto::signed_message::Message::Vote(vote) => {
				let vote = CopperVote::from_proto(vote)?;
				Ok(SignedConsensusMsg::Vote(SignedVote::new(vote, signature)))
			},
		}
	}

	fn encode(&self, msg: &SignedConsensusMsg<ConsensusContext>) -> Result<Bytes, Self::Error> {
		let proto = match msg {
			SignedConsensusMsg::Vote(vote) => proto::SignedMessage {
				message: vote
					.message
					.to_proto()
					.map(proto::signed_message::Message::Vote)
					.map(Some)?,
				signature: Some(codec::proto::encode_signature(&vote.signature)),
			},
			SignedConsensusMsg::Proposal(proposal) => proto::SignedMessage {
				message: Some(proto::signed_message::Message::Proposal(
					proposal.message.to_proto()?,
				)),
				signature: Some(codec::proto::encode_signature(&proposal.signature)),
			},
		};

		Ok(proto.encode_to_vec().into())
	}
}

impl Codec<LivenessMsg<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<LivenessMsg<ConsensusContext>, Self::Error> {
		let msg = proto::LivenessMessage::decode(bytes.as_ref())?;
		match msg.message {
			Some(proto::liveness_message::Message::Vote(vote)) => {
				decode_vote(vote).map(LivenessMsg::Vote)
			},
			Some(proto::liveness_message::Message::PolkaCertificate(cert)) => {
				decode_polka_certificate(cert).map(LivenessMsg::PolkaCertificate)
			},
			Some(proto::liveness_message::Message::RoundCertificate(cert)) => {
				decode_round_certificate(cert).map(LivenessMsg::SkipRoundCertificate)
			},
			None => Err(Error::missing_field::<proto::LivenessMessage>("message")),
		}
	}

	fn encode(&self, msg: &LivenessMsg<ConsensusContext>) -> Result<Bytes, Self::Error> {
		match msg {
			LivenessMsg::Vote(vote) => {
				let message = encode_vote(vote)?;

				Ok(proto::LivenessMessage {
					message: Some(proto::liveness_message::Message::Vote(message)),
				}
				.encode_to_vec()
				.into())
			},
			LivenessMsg::PolkaCertificate(cert) => {
				let message = encode_polka_certificate(cert)?;
				Ok(Bytes::from(
					proto::LivenessMessage {
						message: Some(proto::liveness_message::Message::PolkaCertificate(message)),
					}
					.encode_to_vec(),
				))
			},
			LivenessMsg::SkipRoundCertificate(cert) => {
				let message = encode_round_certificate(cert)?;
				Ok(Bytes::from(
					proto::LivenessMessage {
						message: Some(proto::liveness_message::Message::RoundCertificate(message)),
					}
					.encode_to_vec(),
				))
			},
		}
	}
}

impl Codec<StreamMessage<CopperProposalPart>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<StreamMessage<CopperProposalPart>, Self::Error> {
		let proto = proto::StreamMessage::decode(bytes.as_ref())?;

		let proto_content =
			proto.content.ok_or("content").map_err(Error::missing_field::<proto::StreamMessage>)?;

		let content = match proto_content {
			proto::stream_message::Content::Data(data) => {
				CopperProposalPart::from_bytes(&data).map(StreamContent::Data)?
			},
			proto::stream_message::Content::Fin(_) => StreamContent::Fin,
		};

		Ok(StreamMessage {
			stream_id: StreamId::new(proto.stream_id),
			sequence: proto.sequence,
			content,
		})
	}

	fn encode(&self, msg: &StreamMessage<CopperProposalPart>) -> Result<Bytes, Self::Error> {
		let proto = proto::StreamMessage {
			stream_id: msg.stream_id.to_bytes(),
			sequence: msg.sequence,
			content: match &msg.content {
				StreamContent::Data(data) => {
					data.to_bytes().map(proto::stream_message::Content::Data).map(Some)?
				},
				StreamContent::Fin => Some(proto::stream_message::Content::Fin(true)),
			},
		};

		Ok(Bytes::from(proto.encode_to_vec()))
	}
}

impl Codec<ProposedValue<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<ProposedValue<ConsensusContext>, Self::Error> {
		let proto = proto::ProposedValue::decode(bytes.as_ref())?;

		let proposer = proto
			.proposer
			.ok_or("proposer")
			.map_err(Error::missing_field::<proto::ProposedValue>)?;

		let value =
			proto.value.ok_or("value").map_err(Error::missing_field::<proto::ProposedValue>)?;

		Ok(ProposedValue {
			height: ConsensusHeight::new(proto.height),
			round: Round::new(proto.round),
			valid_round: proto.valid_round.map(Round::new).unwrap_or(Round::Nil),
			proposer: ValidatorAddress::from_proto(proposer)?,
			value: CopperValue::from_proto(value)?,
			validity: Validity::from_bool(proto.validity),
		})
	}

	fn encode(&self, msg: &ProposedValue<ConsensusContext>) -> Result<Bytes, Self::Error> {
		let proto = proto::ProposedValue {
			height: msg.height.as_u64(),
			round: msg.round.as_u32().unwrap(),
			valid_round: msg.valid_round.as_u32(),
			proposer: Some(msg.proposer.to_proto()?),
			value: msg.value.to_proto().map(Some)?,
			validity: msg.validity.to_bool(),
		};

		Ok(proto.encode_to_vec().into())
	}
}

impl Codec<sync::Status<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<sync::Status<ConsensusContext>, Self::Error> {
		let proto = proto::Status::decode(bytes.as_ref())?;

		let proto_peer_id =
			proto.peer_id.ok_or("peer_id").map_err(Error::missing_field::<proto::Status>)?;

		Ok(sync::Status {
			peer_id: PeerId::from_bytes(proto_peer_id.id.as_ref())
				.map_err(|e| Error::Other(e.to_string()))?,
			tip_height: ConsensusHeight::new(proto.height),
			history_min_height: ConsensusHeight::new(proto.earliest_height),
		})
	}

	fn encode(&self, msg: &sync::Status<ConsensusContext>) -> Result<Bytes, Self::Error> {
		let proto = proto::Status {
			peer_id: Some(proto::PeerId { id: Bytes::from(msg.peer_id.to_bytes()) }),
			height: msg.tip_height.as_u64(),
			earliest_height: msg.history_min_height.as_u64(),
		};

		Ok(proto.encode_to_vec().into())
	}
}

impl Codec<sync::Request<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<sync::Request<ConsensusContext>, Self::Error> {
		let proto = proto::SyncRequest::decode(bytes.as_ref())?;
		let request =
			proto.request.ok_or("request").map_err(Error::missing_field::<proto::SyncRequest>)?;

		match request {
			proto::sync_request::Request::ValueRequest(req) => Ok(sync::Request::ValueRequest(
				malachitebft_sync::ValueRequest::new(ConsensusHeight::new(req.height)),
			)),
		}
	}

	fn encode(&self, msg: &sync::Request<ConsensusContext>) -> Result<Bytes, Self::Error> {
		let proto = match msg {
			sync::Request::ValueRequest(req) => proto::SyncRequest {
				request: Some(proto::sync_request::Request::ValueRequest(
					proto::ValueRequest { height: req.height.as_u64() },
				)),
			},
		};

		Ok(proto.encode_to_vec().into())
	}
}

impl Codec<sync::Response<ConsensusContext>> for ProtobufCodec {
	type Error = Error;

	fn decode(&self, bytes: Bytes) -> Result<sync::Response<ConsensusContext>, Self::Error> {
		decode_sync_response(proto::SyncResponse::decode(bytes)?)
	}

	fn encode(&self, response: &sync::Response<ConsensusContext>) -> Result<Bytes, Self::Error> {
		encode_sync_response(response).map(|proto| proto.encode_to_vec().into())
	}
}

fn decode_vote(msg: proto::SignedMessage) -> Result<SignedVote<ConsensusContext>, Error> {
	let signature =
		msg.signature.ok_or("signature").map_err(Error::missing_field::<proto::SignedMessage>)?;

	let vote = match msg.message {
		Some(proto::signed_message::Message::Vote(v)) => v,
		_ => Err(Error::Other("invalid message type: not a vote".into()))?,
	};

	let signature = codec::proto::decode_signature(signature)?;
	let vote = CopperVote::from_proto(vote)?;

	Ok(SignedVote::new(vote, signature))
}

fn encode_polka_certificate(
	polka_certificate: &PolkaCertificate<ConsensusContext>,
) -> Result<proto::PolkaCertificate, Error> {
	Ok(proto::PolkaCertificate {
		height: polka_certificate.height.as_u64(),
		round: polka_certificate.round.as_u32().expect("round should not be nil"),
		value_id: polka_certificate.value_id.to_proto().map(Some)?,
		signatures: polka_certificate
			.polka_signatures
			.iter()
			.map(|sig| -> Result<proto::PolkaSignature, Error> {
				let validator_address = sig.address.to_proto().map(Some)?;
				let signature = Some(codec::proto::encode_signature(&sig.signature));

				Ok(proto::PolkaSignature { validator_address, signature })
			})
			.collect::<Result<Vec<_>, _>>()?,
	})
}

fn decode_polka_certificate(
	certificate: proto::PolkaCertificate,
) -> Result<PolkaCertificate<ConsensusContext>, Error> {
	let value_id = certificate
		.value_id
		.ok_or("value_id")
		.map_err(Error::missing_field::<proto::PolkaCertificate>)
		.and_then(CopperValueId::from_proto)?;

	Ok(PolkaCertificate {
		height: ConsensusHeight::new(certificate.height),
		round: Round::new(certificate.round),
		value_id,
		polka_signatures: certificate
			.signatures
			.into_iter()
			.map(|sig| -> Result<PolkaSignature<ConsensusContext>, Error> {
				let address = sig
					.validator_address
					.ok_or("validator_address")
					.map_err(Error::missing_field::<proto::PolkaCertificate>)?;

				let signature = sig
					.signature
					.ok_or("signature")
					.map_err(Error::missing_field::<proto::PolkaCertificate>)?;

				let signature = codec::proto::decode_signature(signature)?;
				let address = ValidatorAddress::from_proto(address)?;

				Ok(PolkaSignature::new(address, signature))
			})
			.collect::<Result<Vec<_>, _>>()?,
	})
}

fn encode_round_certificate(
	certificate: &RoundCertificate<ConsensusContext>,
) -> Result<proto::RoundCertificate, Error> {
	Ok(proto::RoundCertificate {
		height: certificate.height.as_u64(),
		round: certificate.round.as_u32().expect("round should not be nil"),
		cert_type: match certificate.cert_type {
			RoundCertificateType::Precommit => {
				proto::RoundCertificateType::RoundCertPrecommit.into()
			},
			RoundCertificateType::Skip => proto::RoundCertificateType::RoundCertSkip.into(),
		},
		signatures: certificate
			.round_signatures
			.iter()
			.map(|sig| -> Result<proto::RoundSignature, Error> {
				let value_id = match sig.value_id.clone() {
					NilOrVal::Val(value_id) => value_id.to_proto().map(Some)?,
					NilOrVal::Nil => None,
				};
				Ok(proto::RoundSignature {
					vote_type: malachitebft_test::encode_votetype(sig.vote_type).into(),
					validator_address: Some(sig.address.to_proto()?),
					signature: Some(codec::proto::encode_signature(&sig.signature)),
					value_id,
				})
			})
			.collect::<Result<Vec<_>, _>>()?,
	})
}

fn decode_round_certificate(
	certificate: proto::RoundCertificate,
) -> Result<RoundCertificate<ConsensusContext>, Error> {
	Ok(RoundCertificate {
		height: ConsensusHeight::new(certificate.height),
		round: Round::new(certificate.round),
		cert_type: match proto::RoundCertificateType::try_from(certificate.cert_type)
			.map_err(|_| "unknown RoundCertificateType".into())
			.map_err(Error::Other)?
		{
			proto::RoundCertificateType::RoundCertPrecommit => RoundCertificateType::Precommit,
			proto::RoundCertificateType::RoundCertSkip => RoundCertificateType::Skip,
		},
		round_signatures: certificate
			.signatures
			.into_iter()
			.map(|sig| -> Result<RoundSignature<ConsensusContext>, Error> {
				let vote_type = malachitebft_test::decode_votetype(sig.vote_type());
				let address = sig
					.validator_address
					.ok_or("validator_address")
					.map_err(Error::missing_field::<proto::RoundCertificate>)?;

				let signature = sig
					.signature
					.ok_or("signature")
					.map_err(Error::missing_field::<proto::RoundCertificate>)?;

				let value_id = match sig.value_id {
					Some(value_id) => CopperValueId::from_proto(value_id).map(NilOrVal::Val)?,
					None => NilOrVal::Nil,
				};

				let signature = codec::proto::decode_signature(signature)?;
				let address = ValidatorAddress::from_proto(address)?;

				Ok(RoundSignature::new(vote_type, value_id, address, signature))
			})
			.collect::<Result<Vec<_>, _>>()?,
	})
}

fn encode_vote(vote: &SignedVote<ConsensusContext>) -> Result<proto::SignedMessage, Error> {
	Ok(proto::SignedMessage {
		message: Some(proto::signed_message::Message::Vote(
			vote.message.to_proto()?,
		)),
		signature: Some(codec::proto::encode_signature(&vote.signature)),
	})
}

fn decode_sync_response(
	proto_response: proto::SyncResponse,
) -> Result<sync::Response<ConsensusContext>, Error> {
	let response = proto_response
		.response
		.ok_or("messages")
		.map_err(Error::missing_field::<proto::SyncResponse>)?;

	let response = match response {
		proto::sync_response::Response::ValueResponse(value_response) => {
			sync::Response::ValueResponse(malachitebft_sync::ValueResponse::new(
				ConsensusHeight::new(value_response.height),
				value_response.value.map(decode_synced_value).transpose()?,
			))
		},
	};

	Ok(response)
}

fn encode_sync_response(
	response: &sync::Response<ConsensusContext>,
) -> Result<proto::SyncResponse, Error> {
	let proto = match response {
		sync::Response::ValueResponse(value_response) => proto::SyncResponse {
			response: Some(proto::sync_response::Response::ValueResponse(
				proto::ValueResponse {
					height: value_response.height.as_u64(),
					value: value_response.value.as_ref().map(encode_synced_value).transpose()?,
				},
			)),
		},
	};

	Ok(proto)
}

fn encode_synced_value(
	synced_value: &sync::RawDecidedValue<ConsensusContext>,
) -> Result<proto::SyncedValue, Error> {
	Ok(proto::SyncedValue {
		value_bytes: synced_value.value_bytes.clone(),
		certificate: Some(encode_commit_certificate(&synced_value.certificate)?),
	})
}

fn decode_synced_value(
	proto: proto::SyncedValue,
) -> Result<sync::RawDecidedValue<ConsensusContext>, Error> {
	let certificate = proto
		.certificate
		.ok_or("certificate")
		.map_err(Error::missing_field::<proto::SyncedValue>)
		.and_then(decode_commit_certificate)?;

	Ok(sync::RawDecidedValue { value_bytes: proto.value_bytes, certificate })
}

fn decode_commit_certificate(
	certificate: proto::CommitCertificate,
) -> Result<CommitCertificate<ConsensusContext>, Error> {
	let value_id = certificate
		.value_id
		.ok_or("value_id")
		.map_err(Error::missing_field::<proto::CommitCertificate>)
		.and_then(CopperValueId::from_proto)?;

	let commit_signatures = certificate
		.signatures
		.into_iter()
		.map(|sig| -> Result<CommitSignature<ConsensusContext>, Error> {
			let address = sig.validator_address.ok_or_else(|| {
				Error::missing_field::<proto::CommitCertificate>("validator_address")
			})?;
			let signature = sig
				.signature
				.ok_or_else(|| Error::missing_field::<proto::CommitCertificate>("signature"))?;
			let signature = codec::proto::decode_signature(signature)?;
			let address = ValidatorAddress::from_proto(address)?;

			Ok(CommitSignature::new(address, signature))
		})
		.collect::<Result<Vec<_>, _>>()?;

	let certificate = CommitCertificate {
		height: ConsensusHeight::new(certificate.height),
		round: Round::new(certificate.round),
		value_id,
		commit_signatures,
	};

	Ok(certificate)
}

fn encode_commit_certificate(
	certificate: &CommitCertificate<ConsensusContext>,
) -> Result<proto::CommitCertificate, Error> {
	Ok(proto::CommitCertificate {
		height: certificate.height.as_u64(),
		round: certificate.round.as_u32().expect("round should not be nil"),
		value_id: Some(certificate.value_id.to_proto()?),
		signatures: certificate
			.commit_signatures
			.iter()
			.map(|sig| -> Result<proto::CommitSignature, Error> {
				let address = sig.address.to_proto()?;
				let signature = codec::proto::encode_signature(&sig.signature);
				Ok(proto::CommitSignature {
					validator_address: Some(address),
					signature: Some(signature),
				})
			})
			.collect::<Result<Vec<_>, _>>()?,
	})
}
