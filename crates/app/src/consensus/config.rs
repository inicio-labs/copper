use std::path::Path;

use malachitebft_app_channel::app::node::NodeConfig;
use malachitebft_test_cli::config::{
	ConsensusConfig, LoggingConfig, MetricsConfig, RuntimeConfig, ValueSyncConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
	/// A custom human-readable name for this node
	pub moniker: String,

	/// Log configuration options
	pub logging: LoggingConfig,

	/// Consensus configuration options
	pub consensus: ConsensusConfig,

	/// ValueSync configuration options
	pub value_sync: ValueSyncConfig,

	/// Metrics configuration options
	pub metrics: MetricsConfig,

	/// Runtime configuration options
	pub runtime: RuntimeConfig,
}

impl NodeConfig for Config {
	fn moniker(&self) -> &str {
		&self.moniker
	}

	fn consensus(&self) -> &ConsensusConfig {
		&self.consensus
	}

	fn value_sync(&self) -> &ValueSyncConfig {
		&self.value_sync
	}
}

pub fn load_config<P>(path: P, prefix: Option<&str>) -> eyre::Result<Config>
where
	P: AsRef<Path>,
{
	config::Config::builder()
		.add_source(config::File::from(path.as_ref()))
		.add_source(config::Environment::with_prefix(prefix.unwrap_or("MALACHITE")).separator("__"))
		.build()?
		.try_deserialize()
		.map_err(Into::into)
}
