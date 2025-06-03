use std::error::Error;

pub trait KVStore<E: Error> {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<u8>, E>;

	fn has(&self, key: &Vec<u8>) -> Result<bool, E>;

	fn set(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), E>;

	fn delete(&self, key: &Vec<u8>) -> Result<(), E>;

	fn iterator(
		&self,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, E>;

	fn reverse_iterator(
		&self,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, E>;
}
