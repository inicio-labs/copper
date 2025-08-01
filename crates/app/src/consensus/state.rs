mod error;

use borsh::BorshSerialize;
use copper_base::{
	block::{Block, BlockHeader},
	coin::Coin,
	msg::Msg,
	tx::{SignerInfo, Tx},
};
use copper_facet_bank::types::CoinSend;
use copper_store::iavl::IavlStore;
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use eyre::ContextCompat;
use iavl::kvstore::redb::RedbStore;
use malachitebft_app_channel::app::{
	streaming::{StreamContent, StreamId, StreamMessage},
	types::{LocallyProposedValue, PeerId, ProposedValue, core::Round},
};
use malachitebft_core_types::{CommitCertificate, Validity};
use malachitebft_test::ProposalFin;
use sha2::{Digest, Sha256};

use crate::{App, Registered};

use super::{
	context::ConsensusContext,
	signing::Ed25519Provider,
	store::{ConsensusStore, DecidedValue},
	streaming::{CopperProposalPart, CopperProposalParts, PartStreamMap, ProposalInit},
	types::{ConsensusHeight, CopperGenesis, CopperValidatorSet, CopperValue, ValidatorAddress},
};

use self::error::SignatureVerificationError;

pub struct State<'f> {
	pub current_height: ConsensusHeight,
	pub current_round: Round,
	pub current_proposer: Option<ValidatorAddress>,
	pub store: ConsensusStore,

	signing_provider: Ed25519Provider,
	genesis: CopperGenesis,
	streams_map: PartStreamMap,
	address: ValidatorAddress,
	app: App<'f, IavlStore<RedbStore>, Registered<'f>>,
}

impl<'f> State<'f> {
	pub fn new(
		signing_provider: Ed25519Provider,
		genesis: CopperGenesis,
		address: ValidatorAddress,
		height: ConsensusHeight,
		store: ConsensusStore,
		app: App<'f, IavlStore<RedbStore>, Registered<'f>>,
	) -> Self {
		Self {
			current_height: height,
			current_round: Round::new(0),
			current_proposer: None,
			streams_map: PartStreamMap::new(),
			signing_provider,
			genesis,
			address,
			store,
			app,
		}
	}

	pub async fn get_earliest_height(&self) -> eyre::Result<ConsensusHeight> {
		self.store.min_decided_value_height().await.map(Option::unwrap_or_default)
	}

	pub async fn received_proposal_part(
		&mut self,
		from: PeerId,
		part: StreamMessage<CopperProposalPart>,
	) -> eyre::Result<Option<ProposedValue<ConsensusContext>>> {
		let sequence = part.sequence;

		let Some(parts) = self.streams_map.insert(from, part) else {
			return Ok(None);
		};

		if parts.height < self.current_height {
			tracing::debug!(
				height = %self.current_height,
				round = %self.current_round,
				part.height = %parts.height,
				part.round = %parts.round,
				part.sequence = %sequence,
				"received outdated proposal part, ignoring",
			);

			return Ok(None);
		}

		if let Err(err) = self.verify_proposal_signature(&parts) {
			match err {
				e @ SignatureVerificationError::MissingInitPart
				| e @ SignatureVerificationError::MissingFinPart => eyre::bail!(e),
				e @ SignatureVerificationError::ProposerNotFound
				| e @ SignatureVerificationError::InvalidSignature => {
					tracing::error!("{e}");
					return Ok(None);
				},
			}
		}

		let proposed_value = assemble_value_from_parts(parts)?;

		tracing::info!(
			"storing undecided proposal {} {}",
			proposed_value.height,
			proposed_value.round,
		);

		self.store.store_undecided_proposal(proposed_value.clone()).await?;

		Ok(Some(proposed_value))
	}

	pub async fn get_decided_value(
		&self,
		height: ConsensusHeight,
	) -> eyre::Result<Option<DecidedValue>> {
		self.store.get_decided_value(height).await
	}

	pub fn get_validator_set(&self, height: ConsensusHeight) -> Option<CopperValidatorSet> {
		let num_validators = self.genesis.validator_set.len();
		let selection_size = num_validators.div_ceil(2);

		if num_validators <= selection_size {
			return Some(self.genesis.validator_set.clone());
		}

		let vals = self
			.genesis
			.validator_set
			.iter()
			.cycle()
			.skip(height.as_u64() as usize % num_validators)
			.take(selection_size)
			.cloned()
			.collect();

		CopperValidatorSet::new(vals)
	}

	pub async fn propose_value(
		&mut self,
		height: ConsensusHeight,
		round: Round,
	) -> eyre::Result<LocallyProposedValue<ConsensusContext>> {
		assert_eq!(height, self.current_height);
		assert_eq!(round, self.current_round);

		let proposal = self.create_proposal(height, round).await?;

		Ok(LocallyProposedValue::new(
			proposal.height,
			proposal.round,
			proposal.value,
		))
	}

	pub async fn get_previously_built_value(
		&self,
		height: ConsensusHeight,
		round: Round,
	) -> eyre::Result<Option<LocallyProposedValue<ConsensusContext>>> {
		let proposals = self.store.get_undecided_proposals(height, round).await?;

		assert!(
			proposals.len() <= 1,
			"there should be at most one proposal for a given height and round"
		);

		proposals
			.first()
			.map(|p| LocallyProposedValue::new(p.height, p.round, p.value.clone()))
			.map(Some)
			.map(Ok)
			.unwrap_or(Ok(None))
	}

	async fn create_proposal(
		&mut self,
		height: ConsensusHeight,
		round: Round,
	) -> eyre::Result<ProposedValue<ConsensusContext>> {
		assert_eq!(height, self.current_height);
		assert_eq!(round, self.current_round);

		tracing::error!("PROPOSING at height {height} and round {round}");

		let value = self.make_value(height, round)?;

		let proposal = ProposedValue {
			height,
			round,
			valid_round: Round::Nil,
			proposer: self.address,
			value,
			validity: Validity::Valid,
		};

		self.store.store_undecided_proposal(proposal.clone()).await?;

		Ok(proposal)
	}

	pub async fn commit(
		&mut self,
		certificate: CommitCertificate<ConsensusContext>,
	) -> eyre::Result<()> {
		let (height, round, value_id) = (
			certificate.height,
			certificate.round,
			certificate.value_id.clone(),
		);

		tracing::info!("COMMITTING at height {height} and round {round} the value {value_id}");

		let Ok(Some(proposal)) = self.store.get_undecided_proposal_by_value_id(value_id).await
		else {
			eyre::bail!("no proposal at height {height} and round {round}");
		};

		let _ = self
			.app
			.process_block(proposal.value.get())
			.inspect_err(|e| tracing::error!("failed to process block: {e}"));

		{
			let address = const_hex::decode_to_array("d3d10a83a6e9bdadfff99e3960298e8630e36048")?;

			let account =
				self.app.account_facet().keeper().get_account(&self.app.store, &address)?;

			tracing::error!("ACCOUNT = {account:?}");
		}

		self.store.store_decided_value(certificate, proposal.value).await?;

		self.current_height = self.current_height.increment();
		self.current_round = Round::Nil;

		Ok(())
	}

	pub fn stream_proposal(
		&mut self,
		value: LocallyProposedValue<ConsensusContext>,
		pol_round: Round,
	) -> impl Iterator<Item = StreamMessage<CopperProposalPart>> {
		let parts = self.value_to_parts(value, pol_round);
		let stream_id = self.stream_id();

		let mut msgs = Vec::with_capacity(parts.len() + 1);
		let mut sequence = 0;

		for part in parts {
			let msg = StreamMessage::new(stream_id.clone(), sequence, StreamContent::Data(part));
			sequence += 1;
			msgs.push(msg);
		}

		msgs.push(StreamMessage::new(stream_id, sequence, StreamContent::Fin));

		msgs.into_iter()
	}

	fn stream_id(&self) -> StreamId {
		let mut bytes = Vec::with_capacity(size_of::<u64>() + size_of::<u32>());
		bytes.extend_from_slice(&self.current_height.as_u64().to_be_bytes());
		bytes.extend_from_slice(&self.current_round.as_u32().unwrap().to_be_bytes());
		StreamId::new(bytes.into())
	}

	fn make_value(&mut self, height: ConsensusHeight, _round: Round) -> eyre::Result<CopperValue> {
		let mut alice_sk = {
			let sk_bytes = const_hex::decode_to_array(
				"02360086918045981f7436366a171c6ea2943e0b42dc7d07336f0710c6b3a95e",
			)?;

			SigningKey::from_bytes(&sk_bytes)
		};

		let alice_vk = alice_sk.verifying_key();

		let msg_one = {
			let coin = Coin::new("mudra".parse()?, 40);
			let from_address =
				const_hex::decode_to_array("d3d10a83a6e9bdadfff99e3960298e8630e36048")?;
			let to_address =
				const_hex::decode_to_array("c4a932803947092e599705ac7bfe9e4fc7310e64")?;
			let send_coin_msg = CoinSend::new(from_address, to_address, coin);

			send_coin_msg.to_routable_msg()
		};

		let alice_signer_info = SignerInfo::new(
			alice_vk.as_bytes().to_vec().into(),
			height.as_u64() as u128 - 1,
		);

		let tx = Tx::new(vec![msg_one], vec![alice_signer_info]);

		let hash_to_sign = tx.hash_to_sign(b"poc-chain");

		let alice_sig = alice_sk.try_sign(&hash_to_sign)?;

		let signed_tx = tx
			.into_signed(vec![alice_sig.to_vec().into()])
			.map_err(|_| eyre::eyre!("every signer's sign must be present"))?;

		let header = BlockHeader::new(height.as_u64().try_into()?);

		let block = Block::new(header, vec![signed_tx]);

		Ok(CopperValue::new(block))
	}

	fn value_to_parts(
		&self,
		value: LocallyProposedValue<ConsensusContext>,
		pol_round: Round,
	) -> Vec<CopperProposalPart> {
		let mut hasher = Sha256::new();
		let mut parts = vec![];

		// Init
		// Include metadata about the proposal
		{
			parts.push(CopperProposalPart::Init(ProposalInit::new(
				value.height,
				value.round,
				pol_round,
				self.address,
			)));

			hasher.update(value.height.as_u64().to_be_bytes().as_slice());
			hasher.update(value.round.as_i64().to_be_bytes().as_slice());
		}

		// Data
		// Include each prime factor of the value as a separate proposal part
		{
			value.value.get().serialize(&mut hasher).unwrap();

			parts.push(CopperProposalPart::Data(value.value));
		}

		// Fin
		// Sign the hash of the proposal parts
		{
			let hash = hasher.finalize().to_vec();
			let signature = self.signing_provider.sign(&hash);
			parts.push(CopperProposalPart::Fin(ProposalFin::new(signature)));
		}

		parts
	}

	fn verify_proposal_signature(
		&self,
		parts: &CopperProposalParts,
	) -> Result<(), SignatureVerificationError> {
		let mut hasher = Sha256::new();

		let init = parts.init().ok_or(SignatureVerificationError::MissingInitPart)?;

		let fin = parts.fin().ok_or(SignatureVerificationError::MissingFinPart)?;

		let hash = {
			hasher.update(init.height.as_u64().to_be_bytes());
			hasher.update(init.round.as_i64().to_be_bytes());

			for part in parts.parts.iter().filter_map(|part| part.as_data()) {
				part.get().serialize(&mut hasher).unwrap();
			}

			hasher.finalize()
		};

		let validator_set = self
			.get_validator_set(self.current_height)
			.ok_or(SignatureVerificationError::ProposerNotFound)?;

		let proposer = validator_set
			.get_by_address(&parts.proposer)
			.ok_or(SignatureVerificationError::ProposerNotFound)?;

		if !self.signing_provider.verify(&hash, &fin.signature, &proposer.public_key) {
			return Err(SignatureVerificationError::InvalidSignature);
		}

		Ok(())
	}
}

fn assemble_value_from_parts(
	parts: CopperProposalParts,
) -> eyre::Result<ProposedValue<ConsensusContext>> {
	let valid_round = parts.init().context("missing init part")?.pol_round;

	let value = {
		let mut part_iter = parts.parts.into_iter().filter_map(|part| match part {
			CopperProposalPart::Data(value) => Some(value),
			_ => None,
		});

		let value = part_iter.next().context("finished parts must have a value")?;
		if part_iter.next().is_some() {
			eyre::bail!("finished parts must have exactly one value");
		}

		value
	};

	let CopperProposalParts { height, round, proposer, .. } = parts;

	Ok(ProposedValue { height, round, valid_round, proposer, value, validity: Validity::Valid })
}
