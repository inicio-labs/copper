mod store;

#[cfg(test)]
mod mock_store;

pub use store::KVStore;

#[cfg(test)]
pub use mock_store::MockKVStore;
