use borsh::BorshSerialize;
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use nebz::NonEmptyBz;

use crate::{Map, error::CollectionsError};

#[derive(Debug, Clone)]
pub struct Set<'p, T>(Map<'p, T, ()>);

impl<'p, T> Set<'p, T> {
	pub fn new(prefix: NonEmptyBz<&'p [u8]>) -> Self {
		Self(Map::new(prefix))
	}
}

impl<T> Set<'_, T>
where
	T: BorshSerialize,
{
	pub fn has<S>(&self, store: &S, value: &T) -> Result<bool, CollectionsError>
	where
		S: GetKVStore,
	{
		self.0.get(store, value).map(|v| v.is_some())
	}

	pub fn insert<S>(&self, store: &mut S, value: &T) -> Result<bool, CollectionsError>
	where
		S: InsertKVStore,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
		S::Value: From<Vec<u8>>,
	{
		self.0.insert(store, value, &())
	}

	pub fn remove<S>(&self, store: &mut S, value: &T) -> Result<bool, CollectionsError>
	where
		S: RemoveKVStore,
	{
		self.0.remove(store, value)
	}
}
