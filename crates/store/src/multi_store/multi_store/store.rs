use enum_dispatch::enum_dispatch;

use crate::{
	iavl::iavl_store::IAVLStore,
	multi_store::multi_store::StoreError,
	types::{
		commit_kv_store::CommitKVStore,
		committer::Committer,
		kv_store::KVStore,
		query::{Queryable, RequestQuery, ResponseQuery},
		store::{Store as StoreTrait, StoreType, StoreWithInitialVersion},
	},
};

pub enum Store {
	IAVL(IAVLStore),
}

impl CommitKVStore for Store {}

impl StoreWithInitialVersion for Store {
	fn set_initial_version(&mut self, version: i64) {
		match self {
			Store::IAVL(store) => store.set_initial_version(version),
		}
	}
}

impl KVStore for Store {
	fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		match self {
			Store::IAVL(store) => store.get(key),
		}
	}

	fn has(&self, key: &[u8]) -> bool {
		match self {
			Store::IAVL(store) => store.has(key),
		}
	}

	fn set(&mut self, key: &[u8], value: &[u8]) {
		match self {
			Store::IAVL(store) => store.set(key, value),
		}
	}

	fn delete(&mut self, key: &[u8]) {
		match self {
			Store::IAVL(store) => store.delete(key),
		}
	}

	fn iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> {
		match self {
			Store::IAVL(store) => store.iterator(start, end),
		}
	}

	fn reverse_iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> {
		match self {
			Store::IAVL(store) => store.reverse_iterator(start, end),
		}
	}
}

impl Committer for Store {
	fn commit(&mut self) -> crate::types::committer::CommitID {
		match self {
			Store::IAVL(store) => store.commit(),
		}
	}

	fn last_commit_id(&self) -> crate::types::committer::CommitID {
		match self {
			Store::IAVL(store) => store.last_commit_id(),
		}
	}

	fn working_hash(&self) -> Vec<u8> {
		match self {
			Store::IAVL(store) => store.working_hash(),
		}
	}

	fn set_pruning(&mut self, options: crate::types::prunning::PruningOptions) {
		match self {
			Store::IAVL(store) => store.set_pruning(options),
		}
	}

	fn get_pruning(&self) -> crate::types::prunning::PruningOptions {
		match self {
			Store::IAVL(store) => store.get_pruning(),
		}
	}
}

impl StoreTrait for Store {
	fn get_store_type(&self) -> StoreType {
		match self {
			Store::IAVL(store) => store.get_store_type(),
		}
	}
}

impl Queryable for Store {
	type Error = StoreError;

	fn query(&self, req: &RequestQuery) -> Result<ResponseQuery, Self::Error> {
		match self {
			Store::IAVL(store) => store.query(req),
		}
	}
}
