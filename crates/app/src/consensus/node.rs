mod proto_codec;

pub use self::proto_codec::ProtobufCodec;

use core::num::NonZeroU64;

use std::{collections::HashSet, fs, path::PathBuf, time::Duration};

use bytes::Bytes;
use copper_base::{GenesisInitializer, Initialized, coin::Coin};
use copper_facet_account::{AccountFacet, genesis::AccountGenesis};
use copper_facet_bank::{
	BankFacet,
	genesis::{Balance, BankGenesis},
};
use copper_store::{CommitKVStore, iavl::IavlStore};
use ed25519_dalek::SigningKey;
use iavl::kvstore::redb::RedbStore;
use malachitebft_app_channel::{
	AppMsg, Channels, ConsensusMsg, NetworkMsg,
	app::{
		events::{RxEvent, TxEvent},
		// metrics::SharedRegistry,
		node::{
			CanGeneratePrivateKey, CanMakeConfig, CanMakeGenesis, CanMakePrivateKeyFile,
			EngineHandle, MakeConfigSettings, Node, NodeHandle,
		},
		streaming::StreamContent,
		types::{Keypair, LocallyProposedValue, ProposedValue, sync::RawDecidedValue},
	},
};
use malachitebft_core_types::{Context, Height, Round, Validity, Value, VotingPower};
use malachitebft_test::{PrivateKey, PublicKey};
use malachitebft_test_cli::config::{
	ConsensusConfig, LoggingConfig, MetricsConfig, P2pConfig, PubSubProtocol, TimeoutConfig,
	ValuePayload, ValueSyncConfig,
};
use rand::{CryptoRng, RngCore};
use redb::{Database, backends::InMemoryBackend};
use tokio::task::JoinHandle;
use tracing::Instrument;

use crate::{App, Registered, consensus::types::CopperValue, genesis::AppGenesis};

use super::{
	config::Config,
	context::ConsensusContext,
	signing::Ed25519Provider,
	state::State,
	store::ConsensusStore,
	types::{
		ConsensusHeight, CopperGenesis, CopperValidator, CopperValidatorSet, ValidatorAddress,
	},
};

const SLEEP: Duration = Duration::from_secs(0);

#[derive(Clone)]
pub struct CopperNode {
	pub home_dir: PathBuf,
	pub config_file: PathBuf,
	pub genesis_file: PathBuf,
	pub private_key_file: PathBuf,
	pub start_height: Option<ConsensusHeight>,
}

pub struct Handle {
	pub app: JoinHandle<eyre::Result<()>>,
	pub engine: EngineHandle,
	pub tx_event: TxEvent<ConsensusContext>,
}

#[async_trait::async_trait]
impl NodeHandle<ConsensusContext> for Handle {
	fn subscribe(&self) -> RxEvent<ConsensusContext> {
		self.tx_event.subscribe()
	}

	async fn kill(&self, _reason: Option<String>) -> eyre::Result<()> {
		self.engine.actor.kill_and_wait(None).await?;
		self.app.abort();
		self.engine.handle.abort();
		Ok(())
	}
}

#[async_trait::async_trait]
impl Node for CopperNode {
	type Context = ConsensusContext;
	type Config = Config;
	type Genesis = CopperGenesis;
	type PrivateKeyFile = PrivateKey;
	type SigningProvider = Ed25519Provider;
	type NodeHandle = Handle;

	fn get_home_dir(&self) -> PathBuf {
		self.home_dir.to_owned()
	}

	fn load_config(&self) -> eyre::Result<Self::Config> {
		super::config::load_config(&self.config_file, Some("MALACHITE"))
	}

	fn get_address(&self, pk: &PublicKey) -> <Self::Context as Context>::Address {
		ValidatorAddress::from_public_key(pk)
	}

	fn get_public_key(&self, pk: &PrivateKey) -> PublicKey {
		pk.public_key()
	}

	fn get_keypair(&self, pk: PrivateKey) -> Keypair {
		Keypair::ed25519_from_bytes(pk.inner().to_bytes()).unwrap()
	}

	fn load_private_key(&self, file: Self::PrivateKeyFile) -> PrivateKey {
		file
	}

	fn load_private_key_file(&self) -> eyre::Result<Self::PrivateKeyFile> {
		let private_key = fs::read_to_string(&self.private_key_file)?;
		serde_json::from_str(&private_key).map_err(From::from)
	}

	fn get_signing_provider(&self, private_key: PrivateKey) -> Self::SigningProvider {
		Ed25519Provider::new(private_key)
	}

	fn load_genesis(&self) -> eyre::Result<Self::Genesis> {
		let genesis = fs::read_to_string(&self.genesis_file)?;
		serde_json::from_str(&genesis).map_err(Into::into)
	}

	async fn start(&self) -> eyre::Result<Handle> {
		let config = self.load_config()?;

		let span = tracing::error_span!("node", moniker = %config.moniker);
		let _enter = span.enter();

		let private_key_file = self.load_private_key_file()?;
		let private_key = self.load_private_key(private_key_file);
		let public_key = self.get_public_key(&private_key);
		let address = self.get_address(&public_key);
		let signing_provider = self.get_signing_provider(private_key);
		let ctx = ConsensusContext;

		let genesis = self.load_genesis()?;
		let initial_validator_set = genesis.validator_set.clone();

		let (mut channels, engine_handle) = malachitebft_app_channel::start_engine(
			ctx.clone(),
			self.clone(),
			config.clone(),
			ProtobufCodec, // WAL codec
			ProtobufCodec, // Network codec
			self.start_height,
			initial_validator_set,
		)
		.await?;

		let tx_event = channels.events.clone();

		// let registry = SharedRegistry::global().with_moniker(&config.moniker);

		let db_dir = self.get_home_dir().join("db");
		fs::create_dir_all(&db_dir)?;

		let store = ConsensusStore::open(db_dir.join("store.db")).await?;
		let start_height = self.start_height.unwrap_or(Height::INITIAL);

		let span = tracing::error_span!("node", moniker = %config.moniker);
		let app_handle = tokio::spawn(
			async move {
				let db = Database::builder().create_with_backend(InMemoryBackend::new())?;
				let mut app_store = IavlStore::with_redb(db, "poc-store")?;

				let app_genesis = make_app_genesis()?;
				let (account_facet, bank_facet) =
					make_account_and_bank_facets(&mut app_store, &app_genesis)?;

				let app = make_app(
					app_store,
					app_genesis.chain_id.into_bytes().into(),
					&account_facet,
					&bank_facet,
				)?;

				let mut state =
					State::new(signing_provider, genesis, address, start_height, store, app);
				if let Err(e) = run(&mut state, &mut channels).await {
					tracing::error!(%e, "Application error");
				}

				Ok(())
			}
			.instrument(span),
		);

		Ok(Handle { app: app_handle, engine: engine_handle, tx_event })
	}

	async fn run(self) -> eyre::Result<()> {
		self.start().await?.app.await?
	}
}

impl CanMakeGenesis for CopperNode {
	fn make_genesis(&self, validators: Vec<(PublicKey, VotingPower)>) -> Self::Genesis {
		let validators =
			validators.into_iter().map(|(pk, vp)| CopperValidator::new(pk, vp)).collect();

		let validator_set = CopperValidatorSet::new(validators).unwrap();

		CopperGenesis { validator_set }
	}
}

impl CanGeneratePrivateKey for CopperNode {
	fn generate_private_key<R>(&self, rng: R) -> PrivateKey
	where
		R: RngCore + CryptoRng,
	{
		PrivateKey::generate(rng)
	}
}

impl CanMakePrivateKeyFile for CopperNode {
	fn make_private_key_file(&self, private_key: PrivateKey) -> Self::PrivateKeyFile {
		private_key
	}
}

impl CanMakeConfig for CopperNode {
	fn make_config(index: usize, total: usize, settings: MakeConfigSettings) -> Self::Config {
		make_config(index, total, settings)
	}
}

fn make_app<'f>(
	store: IavlStore<RedbStore>,
	chain_id: Bytes,
	account_facet: &'f AccountFacet<Initialized>,
	bank_facet: &'f BankFacet<'static, Initialized>,
) -> eyre::Result<App<'f, IavlStore<RedbStore>, Registered<'f>>> {
	let mut uapp = App::new(store, chain_id);

	uapp.register_facet("account", account_facet).map_err(|e| eyre::eyre!("{e}"))?;
	uapp.register_facet("bank", bank_facet).map_err(|e| eyre::eyre!("{e}"))?;

	Ok(uapp.into_registered(account_facet))
}

fn make_app_genesis() -> eyre::Result<AppGenesis> {
	let alice_address = {
		let sk_bytes = const_hex::decode_to_array(
			"02360086918045981f7436366a171c6ea2943e0b42dc7d07336f0710c6b3a95e",
		)?;

		let alice_vk = SigningKey::from_bytes(&sk_bytes).verifying_key();

		crate::derive_address(&alice_vk)
	};

	let app_genesis = {
		let account_genesis = AccountGenesis::new(vec![]);

		let balance = Balance::new(alice_address, vec![Coin::new("mudra".parse()?, 10_000)]);
		let bank_genesis = BankGenesis::new(vec![balance]);

		AppGenesis::new(
			"poc-chain".into(),
			NonZeroU64::MIN,
			account_genesis,
			bank_genesis,
		)
	};

	Ok(app_genesis)
}

fn make_account_and_bank_facets<'a>(
	store: &mut IavlStore<RedbStore>,
	app_genesis: &AppGenesis,
) -> eyre::Result<(AccountFacet<Initialized>, BankFacet<'a, Initialized>)> {
	let account_facet = AccountFacet::new()
		.init_genesis(store, &app_genesis.account)
		.map_err(|e| eyre::eyre!("{e}"))?;

	let bank_facet =
		BankFacet::new().init_genesis(store, &app_genesis.bank).map_err(|e| eyre::eyre!("{e}"))?;

	store.commit()?;

	Ok((account_facet, bank_facet))
}

/// Generate configuration for node "index" out of "total" number of nodes.
fn make_config(index: usize, total: usize, settings: MakeConfigSettings) -> Config {
	use rand::Rng;
	use rand::seq::IteratorRandom;

	const CONSENSUS_BASE_PORT: usize = 27000;
	const METRICS_BASE_PORT: usize = 29000;

	let consensus_port = CONSENSUS_BASE_PORT + index;
	let metrics_port = METRICS_BASE_PORT + index;

	Config {
		moniker: format!("app-{}", index),
		consensus: ConsensusConfig {
			// Current channel app does not support parts-only value payload properly as Init does not include valid_round
			value_payload: ValuePayload::ProposalAndParts,
			queue_capacity: 100,
			timeouts: TimeoutConfig::default(),
			p2p: P2pConfig {
				protocol: PubSubProtocol::default(),
				listen_addr: settings.transport.multiaddr("127.0.0.1", consensus_port),
				persistent_peers: if settings.discovery.enabled {
					let mut rng = rand::thread_rng();
					let count = if total > 1 {
						rng.gen_range(1..=(total / 2))
					} else {
						0
					};

					let peers: HashSet<_> = (0..total)
						.filter(|j| *j != index)
						.choose_multiple(&mut rng, count)
						.into_iter()
						.collect();

					peers
						.iter()
						.map(|index| {
							settings.transport.multiaddr("127.0.0.1", CONSENSUS_BASE_PORT + index)
						})
						.collect()
				} else {
					(0..total)
						.filter(|j| *j != index)
						.map(|j| settings.transport.multiaddr("127.0.0.1", CONSENSUS_BASE_PORT + j))
						.collect()
				},
				discovery: settings.discovery,
				..Default::default()
			},
		},
		metrics: MetricsConfig {
			enabled: true,
			listen_addr: format!("127.0.0.1:{metrics_port}").parse().unwrap(),
		},
		runtime: settings.runtime,
		logging: LoggingConfig::default(),
		value_sync: ValueSyncConfig::default(),
	}
}

async fn run<'f>(
	state: &mut State<'f>,
	channels: &mut Channels<ConsensusContext>,
) -> eyre::Result<()> {
	while let Some(msg) = channels.consensus.recv().await {
		match msg {
			AppMsg::ConsensusReady { reply } => {
				let start_height = state
					.store
					.max_decided_value_height()
					.await?
					.map(|height| height.increment())
					.unwrap_or_else(|| Height::INITIAL);

				tracing::info!(%start_height, "consensus is ready");

				tokio::time::sleep(SLEEP).await;

				let _ = reply
					.send((start_height, state.get_validator_set(start_height).unwrap()))
					.inspect_err(|_| tracing::error!("failed to send ConsensusReady reply"));
			},
			AppMsg::StartedRound { height, round, proposer, role, reply_value } => {
				tracing::info!(%height, %round, %proposer, ?role, "Started round");

				reload_log_level(round);

				// We can use that opportunity to update our internal state
				state.current_height = height;
				state.current_round = round;
				state.current_proposer = Some(proposer);

				// If we have already built or seen values for this height and round,
				// send them all back to consensus. This may happen when we are restarting after a crash.
				let proposals = state.store.get_undecided_proposals(height, round).await?;
				let _ = reply_value
					.send(proposals)
					.inspect_err(|_| tracing::error!("failed to send undecided proposals"));
			},
			AppMsg::GetValue { height, round, reply, .. } => {
				tracing::info!(%height, %round, "consensus is requesting a value to propose");

				let proposal = match state.get_previously_built_value(height, round).await? {
					Some(proposal) => {
						tracing::info!(value = %proposal.value.id(), "re-using previously built value");
						proposal
					},
					None => {
						tracing::info!("building a new value to propose");
						state.propose_value(height, round).await?
					},
				};

				let _ = reply
					.send(proposal.clone())
					.inspect_err(|_| tracing::error!("failed to send GetValue reply"));

				let pol_round = Round::Nil;

				// Now what's left to do is to break down the value to propose into parts,
				// and send those parts over the network to our peers, for them to re-assemble the full value.
				for stream_message in state.stream_proposal(proposal, pol_round) {
					tracing::info!(%height, %round, "streaming proposal part: {stream_message:?}");

					channels.network.send(NetworkMsg::PublishProposalPart(stream_message)).await?;
				}
			},
			AppMsg::ExtendVote { reply, .. } => {
				let _ = reply
					.send(None)
					.inspect_err(|_| tracing::error!("failed to send ExtendVote reply"));
			},
			AppMsg::VerifyVoteExtension { reply, .. } => {
				let _ = reply
					.send(Ok(()))
					.inspect_err(|_| tracing::error!("failed to send VerifyVoteExtension reply"));
			},
			AppMsg::ReceivedProposalPart { from, part, reply } => {
				let part_type = match &part.content {
					StreamContent::Data(part) => part.get_type(),
					StreamContent::Fin => "end of stream",
				};

				tracing::info!(
					%from,
					%part.sequence,
					part.type = %part_type,
					"received proposal part",
				);

				let proposed_value = state.received_proposal_part(from, part).await?;

				let _ = reply
					.send(proposed_value)
					.inspect_err(|_| tracing::error!("failed to send ReceivedProposalPart reply"));
			},
			AppMsg::GetValidatorSet { height, reply } => {
				let _ = reply
					.send(state.get_validator_set(height))
					.inspect_err(|_| tracing::error!("failed to send GetValidatorSet reply"));
			},
			AppMsg::Decided { certificate, reply, .. } => {
				tracing::info!(
					height = %certificate.height,
					round = %certificate.round,
					value = %certificate.value_id,
					"consensus has decided on value, committing...",
				);

				match state.commit(certificate).await {
					Ok(_) => {
						let _ = reply
							.send(ConsensusMsg::StartHeight(
								state.current_height,
								state.get_validator_set(state.current_height).unwrap(),
							))
							.inspect_err(|_| tracing::error!("failed to send StartHeight reply"));
					},
					Err(_) => {
						tracing::error!(
							"commit failed, restarting height {}",
							state.current_height
						);

						let _ = reply
							.send(ConsensusMsg::RestartHeight(
								state.current_height,
								state.get_validator_set(state.current_height).unwrap(),
							))
							.inspect_err(|_| tracing::error!("failed to send RestartHeight reply"));
					},
				}

				tokio::time::sleep(SLEEP).await;
			},
			AppMsg::ProcessSyncedValue { height, round, proposer, value_bytes, reply } => {
				tracing::info!(%height, %round, "Processing synced value");

				if let Ok(value) = borsh::from_slice(&value_bytes).map(CopperValue::new) {
					let proposed_value = ProposedValue {
						height,
						round,
						valid_round: Round::Nil,
						proposer,
						value,
						validity: Validity::Valid,
					};

					state.store.store_undecided_proposal(proposed_value.clone()).await?;

					let _ = reply.send(Some(proposed_value)).inspect_err(|_| {
						tracing::error!("failed to send ProcessSyncedValue reply");
					});
				} else {
					let _ = reply.send(None).inspect_err(|_| {
						tracing::error!("failed to send ProcessSyncedValue reply");
					});
				}
			},
			AppMsg::GetDecidedValue { height, reply } => {
				tracing::info!(%height, "received sync request for decided value");

				let decided_value = state.get_decided_value(height).await?;
				tracing::info!(%height, "found decided value: {decided_value:?}");

				let raw_decided_value = decided_value
					.map(|decided_value| {
						eyre::Ok(RawDecidedValue {
							value_bytes: borsh::to_vec(decided_value.value.get())?.into(),
							certificate: decided_value.certificate,
						})
					})
					.transpose()?;

				let _ = reply.send(raw_decided_value).inspect_err(|_| {
					tracing::error!("Failed to send GetDecidedValue reply");
				});
			},
			AppMsg::GetHistoryMinHeight { reply } => {
				let min_height = state.get_earliest_height().await?;

				let _ = reply
					.send(min_height)
					.inspect_err(|_| tracing::error!("failed to send GetHistoryMinHeiht reply"));
			},

			AppMsg::RestreamProposal { height, round, valid_round, value_id, .. } => {
				let proposal_round = if valid_round == Round::Nil {
					round
				} else {
					valid_round
				};

				tracing::info!(%height, %proposal_round, "restreaming existing propos*al...");

				let proposal =
					state.store.get_undecided_proposal(height, proposal_round, value_id).await?;

				if let Some(proposal) = proposal {
					let locally_proposed_value =
						LocallyProposedValue { height, round, value: proposal.value };

					for stream_message in state.stream_proposal(locally_proposed_value, valid_round)
					{
						tracing::info!(
							%height,
							%valid_round,
							"publishing proposal part: {stream_message:?}",
						);

						channels
							.network
							.send(NetworkMsg::PublishProposalPart(stream_message))
							.await?;
					}
				}
			},
		}
	}

	Err(eyre::eyre!("consensus channel closed unexpectedly"))
}

fn reload_log_level(round: Round) {
	use malachitebft_test_cli::logging;

	if round.as_i64() > 0 {
		logging::reload(logging::LogLevel::Debug);
	} else {
		logging::reset();
	}
}
