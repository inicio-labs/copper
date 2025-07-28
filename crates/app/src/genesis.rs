use std::{fs, path::Path};

use copper_base::block::BlockHeight;
use copper_facet_account::genesis::AccountGenesis;
use copper_facet_bank::genesis::BankGenesis;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppGenesis {
	pub chain_id: String,
	pub initial_height: BlockHeight,
	pub account: AccountGenesis,
	pub bank: BankGenesis,
}

impl AppGenesis {
	pub fn new(
		chain_id: String,
		initial_height: BlockHeight,
		account: AccountGenesis,
		bank: BankGenesis,
	) -> Self {
		Self { chain_id, initial_height, account, bank }
	}

	pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
		let contents = fs::read_to_string(path)?;
		ron::from_str(&contents).map_err(From::from)
	}

	pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
		let pretty_config =
			PrettyConfig::new().depth_limit(4).separate_tuple_members(true).enumerate_arrays(true);

		let contents = ron::ser::to_string_pretty(self, pretty_config)?;
		fs::write(path, contents)?;

		Ok(())
	}
}
