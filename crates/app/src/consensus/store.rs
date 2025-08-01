mod table;

use std::{io, path::Path, sync::Arc};

use borsh::{BorshDeserialize, BorshSerialize};
use eyre::ContextCompat;
use malachitebft_app_channel::app::types::ProposedValue;
use malachitebft_core_types::{CommitCertificate, CommitSignature, Round, Validity};
use malachitebft_proto::{Error, Protobuf};
use malachitebft_test::{codec, proto};
use prost::Message;
use redb::{Database, ReadableTable, TableDefinition};
use tokio::task;

use super::{
	context::ConsensusContext,
	types::{ConsensusHeight, CopperValue, CopperValueId, ValidatorAddress},
};

use self::table::{HeightKey, UndecidedValueKey};

#[derive(Debug)]
pub struct DecidedValue {
	pub value: CopperValue,
	pub certificate: CommitCertificate<ConsensusContext>,
}

pub struct ConsensusStore {
	db: Arc<Db>,
}

struct Db {
	db: Database,
}

struct CopperProposedValue {
	height: ConsensusHeight,
	round: Round,
	valid_round: Round,
	proposer: ValidatorAddress,
	value: CopperValue,
	validity: Validity,
}

impl ConsensusStore {
	pub async fn open<P>(path: P) -> eyre::Result<Self>
	where
		P: AsRef<Path> + Send + 'static,
	{
		task::spawn_blocking(move || {
			let db = Db::new(path)?;

			db.create_tables()?;

			Ok(Self { db: Arc::new(db) })
		})
		.await?
	}

	pub async fn min_decided_value_height(&self) -> eyre::Result<Option<ConsensusHeight>> {
		let db = Arc::clone(&self.db);
		task::spawn_blocking(move || db.min_decided_value_height()).await?
	}

	pub async fn max_decided_value_height(&self) -> eyre::Result<Option<ConsensusHeight>> {
		let db = Arc::clone(&self.db);
		task::spawn_blocking(move || db.max_decided_value_height()).await?
	}

	pub async fn get_decided_value(
		&self,
		height: ConsensusHeight,
	) -> eyre::Result<Option<DecidedValue>> {
		let db = Arc::clone(&self.db);

		task::spawn_blocking(move || db.get_decided_value(height)).await?
	}

	pub async fn store_decided_value(
		&self,
		certificate: CommitCertificate<ConsensusContext>,
		value: CopperValue,
	) -> eyre::Result<()> {
		let decided_value = DecidedValue { value, certificate };

		let db = Arc::clone(&self.db);

		task::spawn_blocking(move || db.insert_decided_value(decided_value)).await?
	}

	pub async fn store_undecided_proposal(
		&self,
		value: ProposedValue<ConsensusContext>,
	) -> eyre::Result<()> {
		let db = Arc::clone(&self.db);
		task::spawn_blocking(move || db.insert_undecided_proposal(value)).await?
	}

	pub async fn get_undecided_proposal(
		&self,
		height: ConsensusHeight,
		round: Round,
		value_id: CopperValueId,
	) -> eyre::Result<Option<ProposedValue<ConsensusContext>>> {
		let db = Arc::clone(&self.db);
		task::spawn_blocking(move || db.get_undecided_proposal(height, round, value_id)).await?
	}

	pub async fn get_undecided_proposals(
		&self,
		height: ConsensusHeight,
		round: Round,
	) -> eyre::Result<Vec<ProposedValue<ConsensusContext>>> {
		let db = Arc::clone(&self.db);
		tokio::task::spawn_blocking(move || db.get_undecided_proposals(height, round)).await?
	}

	pub async fn get_undecided_proposal_by_value_id(
		&self,
		value_id: CopperValueId,
	) -> eyre::Result<Option<ProposedValue<ConsensusContext>>> {
		let db = Arc::clone(&self.db);
		tokio::task::spawn_blocking(move || db.get_undecided_proposal_by_value_id(&value_id))
			.await?
	}
}

impl Db {
	const DECIDED_VALUES_TABLE: TableDefinition<'_, HeightKey, Vec<u8>> =
		TableDefinition::new("decided_values");

	const CERTIFICATES_TABLE: TableDefinition<'_, HeightKey, Vec<u8>> =
		TableDefinition::new("certificates");

	const UNDECIDED_PROPOSALS_TABLE: TableDefinition<'_, UndecidedValueKey, Vec<u8>> =
		TableDefinition::new("undecided_values");

	fn new<P>(path: P) -> eyre::Result<Self>
	where
		P: AsRef<Path>,
	{
		let db = Database::create(path)?;
		Ok(Self { db })
	}

	fn get_decided_value(&self, height: ConsensusHeight) -> eyre::Result<Option<DecidedValue>> {
		let tx = self.db.begin_read()?;

		let value = tx
			.open_table(Self::DECIDED_VALUES_TABLE)?
			.get(&height)?
			.map(|value| borsh::from_slice(&value.value()))
			.transpose()?
			.map(CopperValue::new);

		let certificate = tx
			.open_table(Self::CERTIFICATES_TABLE)?
			.get(&height)?
			.map(|value| decode_certificate(&value.value()))
			.transpose()?;

		value
			.zip(certificate)
			.map(|(value, certificate)| DecidedValue { value, certificate })
			.map(Ok)
			.transpose()
	}

	fn insert_decided_value(&self, decided_value: DecidedValue) -> eyre::Result<()> {
		let height = ConsensusHeight::new(decided_value.value.get().header().height().get());
		let tx = self.db.begin_write()?;

		let values_bz = borsh::to_vec(decided_value.value.get())?;
		tx.open_table(Self::DECIDED_VALUES_TABLE)?.insert(height, &values_bz)?;

		let encoded_certificate_bz = encode_certificate(&decided_value.certificate)?;
		tx.open_table(Self::CERTIFICATES_TABLE)?.insert(height, encoded_certificate_bz)?;

		tx.commit()?;

		Ok(())
	}

	fn get_undecided_proposal(
		&self,
		height: ConsensusHeight,
		round: Round,
		value_id: CopperValueId,
	) -> eyre::Result<Option<ProposedValue<ConsensusContext>>> {
		let tx = self.db.begin_read()?;
		let table = tx.open_table(Self::UNDECIDED_PROPOSALS_TABLE)?;

		let value = if let Ok(Some(value)) = table.get(&(height, round, value_id)) {
			let bytes = value.value();

			let CopperProposedValue { height, round, valid_round, proposer, value, validity } =
				borsh::from_slice(&bytes)?;

			let proposal = ProposedValue { height, round, valid_round, proposer, value, validity };

			Some(proposal)
		} else {
			None
		};

		Ok(value)
	}

	fn get_undecided_proposals(
		&self,
		height: ConsensusHeight,
		round: Round,
	) -> eyre::Result<Vec<ProposedValue<ConsensusContext>>> {
		let tx = self.db.begin_read()?;
		let table = tx.open_table(Self::UNDECIDED_PROPOSALS_TABLE)?;

		let mut proposals = vec![];
		for result in table.iter()? {
			let (key, value) = result?;
			let (h, r, _) = key.value();

			if h == height && r == round {
				let bytes = value.value();

				let CopperProposedValue { height, round, valid_round, proposer, value, validity } =
					borsh::from_slice(&bytes)?;

				let proposal =
					ProposedValue { height, round, valid_round, proposer, value, validity };

				proposals.push(proposal);
			}
		}

		Ok(proposals)
	}

	fn insert_undecided_proposal(
		&self,
		ProposedValue { height, round, valid_round, proposer, value, validity }: ProposedValue<
			ConsensusContext,
		>,
	) -> eyre::Result<()> {
		let key = (height, round, CopperValueId::new(value.get().hash()));

		let copper_proposed =
			CopperProposedValue { height, round, valid_round, proposer, value, validity };

		let tx = self.db.begin_write()?;
		{
			let mut table = tx.open_table(Self::UNDECIDED_PROPOSALS_TABLE)?;
			table.insert(key, borsh::to_vec(&copper_proposed)?)?;
		}

		tx.commit()?;

		Ok(())
	}

	fn min_decided_value_height(&self) -> eyre::Result<Option<ConsensusHeight>> {
		let tx = self.db.begin_read()?;
		let table = tx.open_table(Self::DECIDED_VALUES_TABLE)?;
		let Some((key, _)) = table.first()? else {
			return Ok(None);
		};

		Ok(Some(key.value()))
	}

	fn max_decided_value_height(&self) -> eyre::Result<Option<ConsensusHeight>> {
		let tx = self.db.begin_read()?;
		let table = tx.open_table(Self::DECIDED_VALUES_TABLE)?;
		let Some((key, _)) = table.last()? else {
			return Ok(None);
		};

		Ok(Some(key.value()))
	}

	fn create_tables(&self) -> eyre::Result<()> {
		let tx = self.db.begin_write()?;

		// Implicitly creates the tables if they do not exist yet
		let _ = tx.open_table(Self::DECIDED_VALUES_TABLE)?;
		let _ = tx.open_table(Self::CERTIFICATES_TABLE)?;
		let _ = tx.open_table(Self::UNDECIDED_PROPOSALS_TABLE)?;

		tx.commit()?;

		Ok(())
	}

	fn get_undecided_proposal_by_value_id(
		&self,
		value_id: &CopperValueId,
	) -> eyre::Result<Option<ProposedValue<ConsensusContext>>> {
		let tx = self.db.begin_read()?;
		let table = tx.open_table(Self::UNDECIDED_PROPOSALS_TABLE)?;

		for result in table.iter()? {
			let (_, value) = result?;

			let CopperProposedValue { height, round, valid_round, proposer, value, validity } =
				borsh::from_slice(&value.value())?;

			if &value.get().hash() == value_id.get() {
				let proposal =
					ProposedValue { height, round, valid_round, proposer, value, validity };

				return Ok(Some(proposal));
			}
		}

		Ok(None)
	}
}

impl BorshSerialize for CopperProposedValue {
	fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
		BorshSerialize::serialize(&self.height.as_u64(), writer)?;

		BorshSerialize::serialize(&self.round, writer)?;

		BorshSerialize::serialize(&self.valid_round, writer)?;

		BorshSerialize::serialize(&self.proposer, writer)?;

		BorshSerialize::serialize(self.value.get(), writer)?;

		BorshSerialize::serialize(&self.validity, writer)?;

		Ok(())
	}
}

impl BorshDeserialize for CopperProposedValue {
	fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
		let height = ConsensusHeight::new(BorshDeserialize::deserialize_reader(reader)?);

		let round = Round::deserialize_reader(reader)?;

		let valid_round = Round::deserialize_reader(reader)?;

		let proposer = BorshDeserialize::deserialize_reader(reader)?;

		let value = BorshDeserialize::deserialize_reader(reader).map(CopperValue::new)?;

		let validity = Validity::deserialize_reader(reader)?;

		Ok(CopperProposedValue { height, round, valid_round, proposer, value, validity })
	}
}

fn decode_certificate(bz: &[u8]) -> eyre::Result<CommitCertificate<ConsensusContext>> {
	let certificate = proto::CommitCertificate::decode(bz)?;

	let value_id = certificate
		.value_id
		.and_then(|vid| vid.value)
		.as_deref()
		.map(TryFrom::try_from)
		.transpose()?
		.map(CopperValueId::new)
		.context("missging value id")?;

	let commit_signatures = certificate
		.signatures
		.into_iter()
		.map(|sig| -> Result<CommitSignature<ConsensusContext>, Error> {
			let address = sig
				.validator_address
				.ok_or("validator_address")
				.map_err(Error::missing_field::<proto::CommitCertificate>)?;

			let signature = sig
				.signature
				.ok_or("signature")
				.map_err(Error::missing_field::<proto::CommitCertificate>)?;

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

fn encode_certificate(certificate: &CommitCertificate<ConsensusContext>) -> Result<Vec<u8>, Error> {
	let certificate = proto::CommitCertificate {
		height: certificate.height.as_u64(),
		round: certificate.round.as_u32().expect("round should not be nil"),
		value_id: certificate.value_id.to_proto().map(Some)?,
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
	};

	Ok(certificate.encode_to_vec())
}
