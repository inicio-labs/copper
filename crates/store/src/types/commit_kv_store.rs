use enum_dispatch::enum_dispatch;

use crate::iavl::iavl_store::IAVLStore;
use crate::multi_store::multi_store::store::Store;
use crate::types::store::StoreWithInitialVersion;

use super::{committer::Committer, kv_store::KVStore};

/// CommitKVStore combines the capabilities of both a KVStore and Committer

pub trait CommitKVStore: KVStore + Committer + StoreWithInitialVersion {
	// Inherits all methods from KVStore and Committer
}
