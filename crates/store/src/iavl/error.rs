use iavl::{GetError, MutableTreeError};

#[cfg(feature = "redb")]
use iavl::kvstore::redb::RedbStoreError;

#[derive(Debug, thiserror::Error)]
pub enum IavlStoreError {
	#[error("get error: {0}")]
	Get(#[from] GetError),

	#[error("mutable tree error: {0}")]
	MutableTree(#[from] MutableTreeError),

	#[cfg(feature = "redb")]
	#[error("redb store error: {0}")]
	RedbStore(#[from] RedbStoreError),
}
