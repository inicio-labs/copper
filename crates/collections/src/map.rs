use core::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use nebz::NonEmptyBz;

use crate::error::CollectionsError;

#[derive(Debug, Clone)]
pub struct Map<'p, K, V> {
	prefix: NonEmptyBz<&'p [u8]>,
	key: PhantomData<K>,
	value: PhantomData<V>,
}

impl<'p, K, V> Map<'p, K, V> {
	pub fn new(prefix: NonEmptyBz<&'p [u8]>) -> Self {
		Self { prefix, key: PhantomData, value: PhantomData }
	}
}

impl<K, V> Map<'_, K, V>
where
	K: BorshSerialize,
{
	pub fn get<S>(&self, store: &S, key: &K) -> Result<Option<V>, CollectionsError>
	where
		S: GetKVStore,
		V: BorshDeserialize,
	{
		crate::key_bz(self.prefix.as_ref(), key)
			.map(|key| store.get(key))?
			.map_err(|_| CollectionsError::Store)?
			.map(NonEmptyBz::into_inner)
			.map(|bz| V::deserialize(&mut bz.as_ref()))
			.transpose()
			.map_err(|_| CollectionsError::Deserialization)
	}

	pub fn insert<S>(&self, store: &mut S, key: &K, value: &V) -> Result<bool, CollectionsError>
	where
		S: InsertKVStore,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
		NonEmptyBz<S::Value>: for<'a> From<NonEmptyBz<&'a [u8]>>,
		V: BorshSerialize,
	{
		let key = crate::key_bz(self.prefix.as_ref(), key)?;

		let value = {
			let mut buf = vec![];
			value.serialize(&mut buf).map_err(|_| CollectionsError::Serialization)?;
			NonEmptyBz::new(buf).ok_or(CollectionsError::Serialization)?
		};

		store
			.insert(key.as_ref_slice().into(), value.as_ref_slice().into())
			.map_err(|_| CollectionsError::Store)
	}

	pub fn remove<S>(&self, store: &mut S, key: &K) -> Result<bool, CollectionsError>
	where
		S: RemoveKVStore,
	{
		crate::key_bz(self.prefix.as_ref(), key)
			.map(|key| store.remove(key))?
			.map_err(|_| CollectionsError::Store)
	}
}
