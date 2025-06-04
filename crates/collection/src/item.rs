use std::sync::Arc;

use crate::{
	codec::{NoKeyCodec, ValueCodec},
	context::Context,
	map::Map,
	schema::SchemaBuilder,
	store::KVStore,
	CollectionError,
};

const ITEM_KEY: i32 = 0;

pub struct Item<V: Clone, VC: ValueCodec<V> + Clone + 'static, C: Context + Clone + 'static> {
	m: Map<i32, V, NoKeyCodec, VC, C>,
}

impl<V: Clone + 'static, VC: ValueCodec<V> + Clone + 'static, C: Context + Clone + 'static>
	Item<V, VC, C>
{
	pub fn new<T: KVStore<C, CollectionError> + Clone>(
		sb: &mut SchemaBuilder<C, T>,
		store_accessor: Arc<Box<dyn KVStore<C, CollectionError>>>,
		prefix: Vec<u8>,
		name: String,
		value_codec: VC,
	) -> Result<Self, CollectionError> {
		let m = Map::new(sb, store_accessor, prefix, name, NoKeyCodec, value_codec)?;
		Ok(Self { m })
	}

	pub fn get(&self, ctx: &C) -> Result<V, CollectionError> {
		self.m.get(ctx, &ITEM_KEY)
	}

	pub fn set(&self, ctx: &C, value: &V) -> Result<(), CollectionError> {
		self.m.set(ctx, &ITEM_KEY, value)
	}

	pub fn remove(&self, ctx: &C) -> Result<(), CollectionError> {
		self.m.remove(ctx, &ITEM_KEY)
	}

	pub fn has(&self, ctx: &C) -> Result<bool, CollectionError> {
		self.m.has(ctx, &ITEM_KEY)
	}
}
