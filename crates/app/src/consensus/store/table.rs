use core::cmp::Ordering;

use copper_base::block::BlockHash;
use malachitebft_app_channel::app::types::core::Round;
use redb::TypeName;

use crate::consensus::types::{ConsensusHeight, CopperValueId};

pub type UndecidedValueKey = (HeightKey, RoundKey, ValueIdKey);

#[derive(Debug, Clone, Copy)]
pub struct HeightKey;

#[derive(Debug, Clone, Copy)]
pub struct RoundKey;

#[derive(Debug, Clone, Copy)]
pub struct ValueIdKey;

impl redb::Value for HeightKey {
	type SelfType<'a> = ConsensusHeight;

	type AsBytes<'a> = [u8; size_of::<u64>()];

	fn fixed_width() -> Option<usize> {
		Some(size_of::<u64>())
	}

	fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
	where
		Self: 'a,
	{
		let height = <u64 as redb::Value>::from_bytes(data);

		ConsensusHeight::new(height)
	}

	fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
	where
		Self: 'a,
		Self: 'b,
	{
		<u64 as redb::Value>::as_bytes(&value.as_u64())
	}

	fn type_name() -> TypeName {
		TypeName::new("Height")
	}
}

impl redb::Key for HeightKey {
	fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
		<u64 as redb::Key>::compare(data1, data2)
	}
}

impl redb::Value for RoundKey {
	type SelfType<'a> = Round;

	type AsBytes<'a> = [u8; size_of::<i64>()];

	fn fixed_width() -> Option<usize> {
		Some(size_of::<i64>())
	}

	fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
	where
		Self: 'a,
	{
		let round = <i64 as redb::Value>::from_bytes(data);
		Round::from(round)
	}

	fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
	where
		Self: 'a,
		Self: 'b,
	{
		<i64 as redb::Value>::as_bytes(&value.as_i64())
	}

	fn type_name() -> TypeName {
		TypeName::new("Round")
	}
}

impl redb::Key for RoundKey {
	fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
		<i64 as redb::Key>::compare(data1, data2)
	}
}

impl redb::Value for ValueIdKey {
	type SelfType<'a> = CopperValueId;

	type AsBytes<'a> = &'a BlockHash;

	fn fixed_width() -> Option<usize> {
		Some(32)
	}

	fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
	where
		Self: 'a,
	{
		data.try_into().map(CopperValueId::new).unwrap()
	}

	fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
	where
		Self: 'a,
		Self: 'b,
	{
		value.get()
	}

	fn type_name() -> TypeName {
		TypeName::new("ValueId")
	}
}

impl redb::Key for ValueIdKey {
	fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
		<BlockHash as redb::Key>::compare(data1, data2)
	}
}
