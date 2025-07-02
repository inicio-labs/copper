// Supporting traits and types
pub trait Database {
	type Error: std::error::Error;
	type Batch: BatchWriter;

	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
	fn new_batch(&self) -> Self::Batch;
	fn with_prefix(&self, prefix: &[u8]) -> Self;
}

pub trait BatchWriter {
	type Error: std::error::Error;

	fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
	fn write_sync(self) -> Result<(), Self::Error>;
}
