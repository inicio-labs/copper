use borsh::{BorshDeserialize, BorshSerialize};
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use nebz::NonEmptyBz;

use crate::{Map, error::CollectionsError};

#[derive(Debug, Clone)]
pub struct Item<'p, T>(Map<'p, (), T>);

impl<'p, T> Item<'p, T> {
	pub fn new(prefix: NonEmptyBz<&'p [u8]>) -> Self {
		Self(Map::new(prefix))
	}
}

impl<T> Item<'_, T> {
	pub fn get<S>(&self, store: &S) -> Result<Option<T>, CollectionsError>
	where
		S: GetKVStore,
		T: BorshDeserialize,
	{
		self.0.get(store, &())
	}

	pub fn set<S>(&self, store: &mut S, value: &T) -> Result<bool, CollectionsError>
	where
		S: InsertKVStore,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
		S::Value: From<Vec<u8>>,
		T: BorshSerialize,
	{
		self.0.insert(store, &(), value)
	}

	pub fn remove<S>(&self, store: &mut S) -> Result<bool, CollectionsError>
	where
		S: RemoveKVStore,
	{
		self.0.remove(store, &())
	}
}
