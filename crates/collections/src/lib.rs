mod error;
mod item;
mod map;
mod set;

pub use self::{item::Item, map::Map, set::Set};

use borsh::BorshSerialize;
use nebz::NonEmptyBz;

use self::error::CollectionsError;

fn key_bz<P, K>(prefix: NonEmptyBz<P>, key: &K) -> Result<NonEmptyBz<Vec<u8>>, CollectionsError>
where
	K: BorshSerialize,
	P: AsRef<[u8]>,
{
	let mut buf = prefix.get().as_ref().to_vec();

	key.serialize(&mut buf).map_err(|_| CollectionsError::Serialization)?;

	// unwrap is safe here because `buf.len() >= prefix.len()`
	NonEmptyBz::new(buf).map(Ok).unwrap()
}
