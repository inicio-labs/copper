use std::sync::Arc;

use crate::{
	codec::U64ValueCodec, context::Context, item::Item, map::Map, schema::SchemaBuilder,
	store::KVStore, CollectionError,
};

const DEFAULT_SEQUENCE_START: u64 = 0;

pub struct Sequence<C: Context + Clone + 'static, KV: KVStore<C, CollectionError> + Clone + 'static>
{
	i: Item<u64, U64ValueCodec, C, KV>,
}

impl<C: Context + Clone + 'static, KV: KVStore<C, CollectionError> + Clone + 'static>
	Sequence<C, KV>
{
	pub fn new(
		sb: &mut SchemaBuilder<C, KV>,
		store_accessor: Arc<KV>,
		prefix: Vec<u8>,
		name: String,
	) -> Result<Self, CollectionError> {
		let i = Item::new(sb, store_accessor, prefix, name, U64ValueCodec)?;
		Ok(Self { i })
	}

	/// Peek returns the current sequence value. If no number is set,
	/// then the DEFAULT_SEQUENCE_START is returned.
	/// Returns an error on encoding issues.
	pub fn peek(&self, ctx: &C) -> Result<u64, CollectionError> {
		match self.i.get(ctx) {
			Ok(n) => Ok(n),
			Err(CollectionError::NotFoundError) => Ok(DEFAULT_SEQUENCE_START),
			Err(e) => Err(e),
		}
	}

	/// Next returns the next sequence number and sets the next expected sequence.
	/// Returns an error on encoding issues.
	pub fn next(&self, ctx: &C) -> Result<u64, CollectionError> {
		let seq = self.peek(ctx)?;
		self.set(ctx, seq + 1)?;
		Ok(seq)
	}

	/// Set hard resets the sequence to the provided value.
	/// Returns an error on encoding issues.
	pub fn set(&self, ctx: &C, value: u64) -> Result<(), CollectionError> {
		self.i.set(ctx, &value as &u64)
	}
}
