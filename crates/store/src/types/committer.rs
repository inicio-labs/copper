use std::fmt::Display;

use crate::multi_store::multi_store::store::Store;
use enum_dispatch::enum_dispatch;

use crate::types::prunning::PruningOptions;

// Supporting types that the traits reference
#[derive(Debug, Clone, Default)]
pub struct CommitID {
	pub version: i64,
	pub hash: Vec<u8>,
}

impl Display for CommitID {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "CommitID{{{}:{:?}}}", self.version, self.hash)
	}
}

/// Committer trait for stores that can persist to disk
pub trait Committer {
	/// Commit the current state and return the commit ID
	fn commit(&mut self) -> CommitID;

	/// Get the last commit ID
	fn last_commit_id(&self) -> CommitID;

	/// Get the working hash before commit
	fn working_hash(&self) -> Vec<u8>;

	/// Set pruning options
	fn set_pruning(&mut self, options: PruningOptions);

	/// Get current pruning options
	fn get_pruning(&self) -> PruningOptions;
}
