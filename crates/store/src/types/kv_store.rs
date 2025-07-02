use enum_dispatch::enum_dispatch;

use super::store::Store;

/// Basic KVStore operations
///

pub trait KVStore: Store {
	/// Get value by key, returns None if key doesn't exist
	fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

	/// Check if key exists
	fn has(&self, key: &[u8]) -> bool;

	/// Set key-value pair
	fn set(&mut self, key: &[u8], value: &[u8]);

	/// Delete key
	fn delete(&mut self, key: &[u8]);

	/// Create iterator over key range (start inclusive, end exclusive)
	fn iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;

	// TODO: Change the Iterator struct later.
	/// Create reverse iterator over key range
	fn reverse_iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;
}
