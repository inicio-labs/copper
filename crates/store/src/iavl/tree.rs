use std::sync::Arc;

use crate::types::committer::CommitID;

// TODO replace it once iavl is ready
pub struct ImmutableTree {}
pub struct MutableTree {}

pub enum Tree {
	Immutable(ImmutableTree),
	Mutable(MutableTree),
}

pub struct ChangeSet {}

/// Tree trait defines the interface for a versioned, merkle tree data structure (IAVL)
/// that supports efficient key-value operations, versioning, and cryptographic proofs.
pub trait TreeTrait {
	type Error;
	type Iterator: Iterator<Item = (Vec<u8>, Vec<u8>)>;

	/// Check if a key exists
	fn has(&self, key: &[u8]) -> Result<bool, Self::Error>;

	/// Get a value by key
	fn get(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;

	/// Set a key-value pair
	fn set(&mut self, key: &[u8], value: &[u8]) -> Result<bool, Self::Error>;

	/// Remove a key and return the old value and whether it existed
	fn remove(&mut self, key: &[u8]) -> Result<(Vec<u8>, bool), Self::Error>;

	/// Save the current version and return hash and version
	fn save_version(&mut self) -> Result<(Vec<u8>, i64), Self::Error>;

	/// Get the current version
	fn version(&self) -> i64;

	/// Get the hash of the current version
	fn hash(&self) -> Vec<u8>;

	/// Get the working hash
	fn working_hash(&self) -> Vec<u8>;

	/// Check if a version exists
	fn version_exists(&self, version: i64) -> bool;

	/// Delete versions up to and including the specified version
	fn delete_versions_to(&mut self, version: i64) -> Result<(), Self::Error>;

	/// Get a value by key at a specific version
	fn get_versioned(&self, key: &[u8], version: i64) -> Result<Vec<u8>, Self::Error>;

	/// Get an immutable tree at a specific version
	fn get_immutable(&self, version: i64) -> Result<Arc<ImmutableTree>, Self::Error>;

	/// Set the initial version
	fn set_initial_version(&mut self, version: u64);

	/// Create an iterator
	fn iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
		ascending: bool,
	) -> Result<Self::Iterator, Self::Error>;

	/// Get available versions
	fn available_versions(&self) -> Vec<i32>;

	/// Load a version for overwriting
	fn load_version_for_overwriting(&mut self, target_version: i64) -> Result<(), Self::Error>;

	/// Traverse state changes between versions
	fn traverse_state_changes<F>(
		&self,
		start_version: i64,
		end_version: i64,
		callback: F,
	) -> Result<(), Self::Error>
	where
		F: Fn(i64, &ChangeSet) -> Result<(), Self::Error>;
}
