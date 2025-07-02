use std::time::SystemTime;

use crate::types::{committer::CommitID, store::StoreInfo};

/// CommitInfo contains information about a commit including version, store infos, and timestamp
#[derive(Debug, Clone)]
pub struct CommitInfo {
	/// Version of the commit
	pub version: i64,
	/// Information about each store in this commit
	pub store_infos: Vec<StoreInfo>,
	/// Timestamp when the commit was made
	pub timestamp: SystemTime,
}

impl Default for CommitInfo {
	fn default() -> Self {
		Self { version: 0, store_infos: Vec::new(), timestamp: SystemTime::now() }
	}
}

impl CommitInfo {
	/// Create a new CommitInfo
	pub fn new(version: i64, store_infos: Vec<StoreInfo>, timestamp: SystemTime) -> Self {
		Self { version, store_infos, timestamp }
	}

	/// Create a new CommitInfo with current timestamp
	pub fn new_with_current_time(version: i64, store_infos: Vec<StoreInfo>) -> Self {
		Self::new(version, store_infos, SystemTime::now())
	}

	/// Hash returns the simple merkle root hash of the stores sorted by name.
	pub fn hash(&self) -> Vec<u8> {
		todo!()
	}

	/// Returns the commit ID with version and hash
	pub fn commit_id(&self) -> CommitID {
		CommitID { version: self.version, hash: self.hash() }
	}

	/// Serialize the CommitInfo to bytes
	pub fn marshal(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
		todo!()
	}

	/// Deserialize CommitInfo from bytes
	pub fn unmarshal(_data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
		todo!()
	}
}
