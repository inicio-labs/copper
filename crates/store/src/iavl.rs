mod error;

use bytes::Bytes;
use iavl::{
	Get, MutableTree,
	kvstore::{KVStore, MutKVStore},
};
use nebz::NonEmptyBz;
use oblux::U63;

#[cfg(feature = "redb")]
use redb::Database;

use crate::{CommitKVStore, GetKVStore, InsertKVStore, RemoveKVStore};

use self::error::IavlStoreError;

pub struct IavlStore<DB> {
	tree: MutableTree<DB>,
}

impl<DB> IavlStore<DB> {
	pub fn new(db: DB) -> Self {
		Self { tree: MutableTree::new(db) }
	}
}

#[cfg(feature = "redb")]
impl IavlStore<iavl::kvstore::redb::RedbStore> {
	pub fn with_redb(db: Database, namespace: &'static str) -> Result<Self, IavlStoreError> {
		let store = iavl::kvstore::redb::RedbStore::new(std::sync::Arc::new(db), namespace)?;

		Ok(Self { tree: MutableTree::new(store) })
	}
}

impl<DB> GetKVStore for IavlStore<DB>
where
	DB: KVStore,
{
	type Value = Bytes;

	type Error = IavlStoreError;

	fn get<K>(&self, key: NonEmptyBz<K>) -> Result<Option<Self::Value>, Self::Error>
	where
		K: AsRef<[u8]>,
	{
		self.tree.get(key).map(|(_, val)| val).map_err(From::from)
	}
}

impl<DB> InsertKVStore for IavlStore<DB>
where
	DB: MutKVStore + KVStore + Clone,
{
	type Key = Bytes;

	type Value = Bytes;

	type Error = IavlStoreError;

	fn insert(
		&mut self,
		key: NonEmptyBz<Self::Key>,
		value: Self::Value,
	) -> Result<bool, Self::Error> {
		self.tree.insert(key, value).map_err(From::from)
	}
}

impl<DB> RemoveKVStore for IavlStore<DB>
where
	DB: MutKVStore + KVStore + Clone,
{
	type Error = IavlStoreError;

	fn remove<K>(&mut self, key: NonEmptyBz<K>) -> Result<bool, Self::Error>
	where
		K: AsRef<[u8]>,
	{
		self.tree.remove(key).map_err(From::from)
	}
}

impl<DB> CommitKVStore for IavlStore<DB>
where
	DB: MutKVStore + KVStore + Clone,
{
	type Error = IavlStoreError;

	fn commit(&mut self) -> Result<U63, Self::Error> {
		self.tree.save().map_err(From::from)
	}
}
