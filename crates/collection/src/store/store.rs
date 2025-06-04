use std::error::Error;

use crate::context::Context;

pub trait KVStore<C: Context, E: Error> {
	fn get(&self, ctx: &C, key: &Vec<u8>) -> Result<Vec<u8>, E>;

	fn has(&self, ctx: &C, key: &Vec<u8>) -> Result<bool, E>;

	fn set(&self, ctx: &C, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), E>;

	fn delete(&self, ctx: &C, key: &Vec<u8>) -> Result<(), E>;

	fn iterator(
		&self,
		ctx: &C,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, E>;

	fn reverse_iterator(
		&self,
		ctx: &C,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, E>;
}
