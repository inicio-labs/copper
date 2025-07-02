use std::sync::{Arc, RwLock};

use enum_dispatch::enum_dispatch;

use crate::{
	iavl::tree::Tree,
	multi_store::multi_store::StoreError,
	types::{
		commit_kv_store::CommitKVStore,
		committer::{CommitID, Committer},
		kv_store::KVStore,
		prunning::PruningOptions,
		query::{self, Queryable, RequestQuery, ResponseQuery},
		store::{Store, StoreType, StoreWithInitialVersion},
	},
};

pub struct IAVLStore {
	tree: Tree,
}

impl IAVLStore {
	/// LoadStore returns an IAVL Store as a CommitKVStore. Internally, it will load the
	/// store's version (id) from the provided DB. An error is returned if the version
	/// fails to load, or if called with a positive version on an empty tree.
	pub fn load_store<DB, SK>(
		_db: DB,
		_key: SK,
		_id: CommitID,
		_cache_size: i32,
		_disable_fast_node: bool,
	) -> Result<IAVLStore, Box<dyn std::error::Error>> {
		todo!()
	}

	/// LoadStoreWithInitialVersion returns an IAVL Store as a CommitKVStore setting its initialVersion
	/// to the one given. Internally, it will load the store's version (id) from the
	/// provided DB. An error is returned if the version fails to load, or if called with a positive
	/// version on an empty tree.
	pub fn load_store_with_initial_version<DB, SK>(
		_db: DB,
		_key: SK,
		_id: CommitID,
		_initial_version: u64,
		_cache_size: i32,
		_disable_fast_node: bool,
	) -> Result<IAVLStore, Box<dyn std::error::Error>> {
		todo!()
	}
}

impl StoreWithInitialVersion for IAVLStore {
	fn set_initial_version(&mut self, version: i64) {
		todo!()
	}
}

impl Store for IAVLStore {
	fn get_store_type(&self) -> StoreType {
		StoreType::IAVL
	}
}

impl KVStore for IAVLStore {
	/// Get value by key, returns None if key doesn't exist
	fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		todo!()
	}

	/// Check if key exists
	fn has(&self, key: &[u8]) -> bool {
		todo!()
	}

	/// Set key-value pair
	fn set(&mut self, key: &[u8], value: &[u8]) {
		todo!()
	}

	/// Delete key
	fn delete(&mut self, key: &[u8]) {
		todo!()
	}

	/// Create iterator over key range (start inclusive, end exclusive)
	fn iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> {
		todo!()
	}

	// TODO: Change the Iterator struct later.
	/// Create reverse iterator over key range
	fn reverse_iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> {
		todo!()
	}
}

impl Committer for IAVLStore {
	/// Commit the current state and return the commit ID
	fn commit(&mut self) -> CommitID {
		todo!()
	}

	/// Get the last commit ID
	fn last_commit_id(&self) -> CommitID {
		todo!()
	}

	/// Get the working hash before commit
	fn working_hash(&self) -> Vec<u8> {
		todo!()
	}

	/// Set pruning options
	fn set_pruning(&mut self, _options: PruningOptions) {
		todo!()
	}

	/// Get current pruning options
	fn get_pruning(&self) -> PruningOptions {
		todo!()
	}
}

impl CommitKVStore for IAVLStore {}

impl Queryable for IAVLStore {
	type Error = StoreError;

	fn query(&self, req: &RequestQuery) -> Result<ResponseQuery, Self::Error> {
		todo!()
	}
}
