use std::sync::Arc;

use crate::{
	codec::{NoKeyCodec, ValueCodec},
	map::Map,
	schema::SchemaBuilder,
	store::KVStore,
	CollectionError,
};

const ITEM_KEY: i32 = 0;

pub struct Item<V: Clone, VC: ValueCodec<V> + Clone + 'static> {
	m: Map<i32, V, NoKeyCodec, VC>,
}

impl<V: Clone + 'static, VC: ValueCodec<V> + Clone + 'static> Item<V, VC> {
	pub fn new<T: KVStore<CollectionError> + Clone>(
		sb: &mut SchemaBuilder<T>,
		store_accessor: Arc<Box<dyn KVStore<CollectionError>>>,
		prefix: Vec<u8>,
		name: String,
		value_codec: VC,
	) -> Result<Self, CollectionError> {
		let m = Map::new(sb, store_accessor, prefix, name, NoKeyCodec, value_codec)?;
		Ok(Self { m })
	}

	pub fn get(&self) -> Result<V, CollectionError> {
		self.m.get(&ITEM_KEY)
	}

	pub fn set(&self, value: &V) -> Result<(), CollectionError> {
		self.m.set(&ITEM_KEY, value)
	}

	pub fn remove(&self, key: &i32) -> Result<(), CollectionError> {
		self.m.remove(&ITEM_KEY)
	}

	pub fn has(&self) -> Result<bool, CollectionError> {
		self.m.has(&ITEM_KEY)
	}
}
