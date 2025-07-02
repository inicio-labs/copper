use std::fmt::{Debug, Display};

use enum_dispatch::enum_dispatch;

use crate::{
	multi_store::multi_store::{Database, store::Store as ExposedStore},
	types::store_key::StoreKey,
};

use super::cache::CacheWrap;

/// Parameters for configuring individual stores
#[derive(Debug, Clone)]
pub struct StoreParams<DB: Database, SK: StoreKey> {
	/// Store key identifier
	pub key: SK,
	/// Database instance for this store
	pub db: DB,
	/// Type of store (IAVL, Memory, Transient, etc.)
	pub store_type: StoreType,
	/// Initial version when the store was created
	pub initial_version: u64,
}

impl<DB: Database, SK: StoreKey> StoreParams<DB, SK> {
	pub fn new(db: DB, key: SK, store_type: StoreType, initial_version: u64) -> Self {
		Self { key, db, store_type, initial_version }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreType {
	MultiStore,
	IAVL,
	DB,
	Transient,
	Memory,
	SMT,
}

impl Display for StoreType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StoreType::MultiStore => write!(f, "StoreTypeMulti"),
			StoreType::DB => write!(f, "StoreTypeDB"),
			StoreType::IAVL => write!(f, "StoreTypeIAVL"),
			StoreType::Transient => write!(f, "StoreTypeTransient"),
			StoreType::Memory => write!(f, "StoreTypeMemory"),
			StoreType::SMT => write!(f, "StoreTypeSMT"),
		}
	}
}

/// StoreInfo contains information about an individual store in a commit
#[derive(Debug, Clone)]
pub struct StoreInfo {
	/// Name/identifier of the store
	pub name: String,
	/// Hash of the store's state at this commit
	pub commit_id: super::committer::CommitID,
}

impl StoreInfo {
	/// Create a new StoreInfo
	pub fn new(name: String, commit_id: super::committer::CommitID) -> Self {
		Self { name, commit_id }
	}
}

/// Base Store trait
pub trait Store {
	/// Get the type of this store
	fn get_store_type(&self) -> StoreType;
}

// StoreWithInitialVersion is a store that can have an arbitrary initial
// version.
pub trait StoreWithInitialVersion {
	// SetInitialVersion sets the initial version of the IAVL tree. It is used when
	// starting a new chain at an arbitrary height.
	fn set_initial_version(&mut self, version: i64);
}
