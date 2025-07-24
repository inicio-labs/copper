#[cfg(feature = "iavl")]
pub mod iavl;

use nebz::NonEmptyBz;
use oblux::U63;

pub trait GetKVStore {
	type Value: AsRef<[u8]>;

	type Error;

	fn get<K>(&self, key: NonEmptyBz<K>) -> Result<Option<Self::Value>, Self::Error>
	where
		K: AsRef<[u8]>;
}

pub trait InsertKVStore {
	type Key: AsRef<[u8]>;

	type Value: AsRef<[u8]>;

	type Error;

	fn insert(
		&mut self,
		key: NonEmptyBz<Self::Key>,
		value: Self::Value,
	) -> Result<bool, Self::Error>;
}

pub trait RemoveKVStore {
	type Error;

	fn remove<K>(&mut self, key: NonEmptyBz<K>) -> Result<bool, Self::Error>
	where
		K: AsRef<[u8]>;
}

pub trait CommitKVStore {
	type Error;

	fn commit(&mut self) -> Result<U63, Self::Error>;
}
