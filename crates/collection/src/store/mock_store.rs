use crate::{store::KVStore, CollectionError};
use std::{collections::HashMap, sync::Arc, sync::Mutex};

#[derive(Clone)]
pub struct MockKVStore {
	data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MockKVStore {
	pub fn new() -> Self {
		Self { data: Arc::new(Mutex::new(HashMap::new())) }
	}

	// Helper method for tests to inspect internal state
	pub fn dump(&self) -> HashMap<Vec<u8>, Vec<u8>> {
		self.data.lock().unwrap().clone()
	}
}

impl KVStore<CollectionError> for MockKVStore {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<u8>, CollectionError> {
		self.data.lock().unwrap().get(key).cloned().ok_or(CollectionError::NotFoundError)
	}

	fn has(&self, key: &Vec<u8>) -> Result<bool, CollectionError> {
		Ok(self.data.lock().unwrap().contains_key(key))
	}

	fn set(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), CollectionError> {
		self.data.lock().unwrap().insert(key.clone(), value.clone());
		Ok(())
	}

	fn delete(&self, key: &Vec<u8>) -> Result<(), CollectionError> {
		self.data.lock().unwrap().remove(key);
		Ok(())
	}

	fn iterator(
		&self,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, CollectionError> {
		let data = self.data.lock().unwrap();
		let mut v = data
			.iter()
			.filter(move |(k, _)| *k >= start && *k < end)
			.map(|(k, v)| (k.clone(), v.clone()))
			.collect::<Vec<_>>();

		v.sort_by(|a, b| a.0.cmp(&b.0));

		let iter = v.into_iter();

		Ok(Box::new(iter))
	}

	fn reverse_iterator(
		&self,
		start: &Vec<u8>,
		end: &Vec<u8>,
	) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, CollectionError> {
		let data = self.data.lock().unwrap();
		let mut v = data
			.iter()
			.filter(move |(k, _)| *k >= start && *k < end)
			.map(|(k, v)| (k.clone(), v.clone()))
			.collect::<Vec<_>>();

		v.sort_by(|a, b| a.0.cmp(&b.0));

		let iter = v.into_iter().rev();

		Ok(Box::new(iter))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_operations() {
		let store = MockKVStore::new();
		let key = b"test_key".to_vec();
		let value = b"test_value".to_vec();

		// Test set
		store.set(&key, &value).unwrap();
		assert!(store.has(&key).unwrap());

		// Test get
		let retrieved = store.get(&key).unwrap();
		assert_eq!(retrieved, value);

		// Test delete
		store.delete(&key).unwrap();
		assert!(!store.has(&key).unwrap());
		assert!(store.get(&key).is_err());
	}

	#[test]
	fn test_iterator() {
		let store = MockKVStore::new();

		// Insert some test data
		store.set(&b"a".to_vec(), &b"1".to_vec()).unwrap();
		store.set(&b"b".to_vec(), &b"2".to_vec()).unwrap();
		store.set(&b"c".to_vec(), &b"3".to_vec()).unwrap();

		// Test forward iterator
		let iter = store.iterator(&b"a".to_vec(), &b"c".to_vec()).unwrap();
		let items: Vec<_> = iter.collect();
		assert_eq!(items.len(), 2); // Should include 'a' and 'b' but not 'c'

		// Test reverse iterator
		let iter = store.reverse_iterator(&b"a".to_vec(), &b"c".to_vec()).unwrap();
		let items: Vec<_> = iter.collect();
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].0, b"b".to_vec()); // First item should be 'b' in reverse order
	}
}
