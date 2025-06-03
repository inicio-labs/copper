use std::marker::PhantomData;
use std::sync::Arc;

use crate::{
	codec::{KeyCodec, ValueCodec},
	collection::Collection,
	ranger::{Direction, Ranger},
	schema::SchemaBuilder,
	store::KVStore,
	CollectionError,
};

#[derive(Clone)]
pub struct Map<
	K: Clone,
	V: Clone,
	KC: KeyCodec<K> + Clone + 'static,
	VC: ValueCodec<V> + Clone + 'static,
> {
	store_accessor: Arc<Box<dyn KVStore<CollectionError>>>,
	prefix: Vec<u8>,
	name: String,
	key_codec: KC,
	value_codec: VC,
	_key_type: PhantomData<K>,
	_value_type: PhantomData<V>,
}

impl<
		K: 'static + Clone,
		V: 'static + Clone,
		KC: KeyCodec<K> + Clone + 'static,
		VC: ValueCodec<V> + Clone + 'static,
	> Map<K, V, KC, VC>
{
	pub fn new<T: KVStore<CollectionError> + Clone>(
		sb: &mut SchemaBuilder<T>,
		store_accessor: Arc<Box<dyn KVStore<CollectionError>>>,
		prefix: Vec<u8>,
		name: String,
		key_codec: KC,
		value_codec: VC,
	) -> Result<Map<K, V, KC, VC>, CollectionError> {
		let m = Self {
			store_accessor: store_accessor.clone(),
			prefix,
			name,
			key_codec,
			value_codec,
			_key_type: PhantomData,
			_value_type: PhantomData,
		};

		let arc_m: Arc<Box<dyn Collection>> = Arc::new(Box::new(m.clone()));

		sb.add_collection(arc_m)?;

		return Ok(m);
	}
}

impl<
		K: Clone,
		V: Clone,
		KC: KeyCodec<K> + Clone + 'static,
		VC: ValueCodec<V> + Clone + 'static,
	> Collection for Map<K, V, KC, VC>
{
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn get_prefix(&self) -> Vec<u8> {
		self.prefix.clone()
	}
}

impl<
		K: Default + Clone,
		V: Clone,
		KC: KeyCodec<K> + Clone + 'static,
		VC: ValueCodec<V> + Clone + 'static,
	> Map<K, V, KC, VC>
{
	pub fn set(&self, key: &K, value: &V) -> Result<(), CollectionError> {
		let key_bytes = encode_key_with_prefix(&self.prefix, key, &self.key_codec)?;
		let value_bytes = self.value_codec.encode(value)?;
		self.store_accessor.set(&key_bytes, &value_bytes)?;
		Ok(())
	}

	pub fn get(&self, key: &K) -> Result<V, CollectionError> {
		let key_bytes = encode_key_with_prefix(&self.prefix, key, &self.key_codec)?;
		let value_bytes = self.store_accessor.get(&key_bytes)?;
		let value = self.value_codec.decode(&value_bytes)?;
		Ok(value)
	}

	pub fn remove(&self, key: &K) -> Result<(), CollectionError> {
		let key_bytes = encode_key_with_prefix(&self.prefix, key, &self.key_codec)?;
		self.store_accessor.delete(&key_bytes)?;
		Ok(())
	}

	pub fn has(&self, key: &K) -> Result<bool, CollectionError> {
		let key_bytes = encode_key_with_prefix(&self.prefix, key, &self.key_codec)?;
		let value_bytes = self.store_accessor.get(&key_bytes)?;
		Ok(value_bytes.len() > 0)
	}

	pub fn iter(
		&self,
		rng: Ranger<K>,
	) -> Result<Box<dyn Iterator<Item = (K, V)>>, CollectionError> {
		let key_codec = self.key_codec.clone();
		let value_codec = self.value_codec.clone();

		let start_key = match rng.start {
			Some(start) => encode_key_with_prefix(&self.prefix, &start, &self.key_codec)?,
			None => encode_key_with_prefix(&self.prefix, &K::default(), &self.key_codec)?,
		};

		let end_key = match rng.end {
			Some(end) => encode_key_with_prefix(&self.prefix, &end, &self.key_codec)?,
			None => encode_key_with_prefix(&self.prefix, &K::default(), &self.key_codec)?,
		};

		let iter = match rng.direction {
			Direction::Asc => self.store_accessor.iterator(&start_key, &end_key)?,
			Direction::Desc => self.store_accessor.reverse_iterator(&start_key, &end_key)?,
		};

		let mapped_iter = iter.map(move |(key, value)| {
			// First try to decode the key
			let key_result = key_codec.decode(&key).unwrap();

			// Then try to decode the value
			let value_result = value_codec.decode(&value).unwrap();

			// If both succeed, return the tuple
			(key_result.1, value_result)
		});

		Ok(Box::new(mapped_iter))
	}
}

fn encode_key_with_prefix<K, KC: KeyCodec<K>>(
	prefix: &Vec<u8>,
	key: &K,
	key_codec: &KC,
) -> Result<Vec<u8>, CollectionError> {
	let mut prefic_key_bytes = Vec::new();
	prefic_key_bytes.extend_from_slice(prefix);

	let res = key_codec.encode(&mut prefic_key_bytes, &key);

	if res.is_err() {
		return Err(res.err().unwrap());
	}

	return Ok(prefic_key_bytes);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		codec::{BytesKeyCodec, BytesValueCodec},
		ranger::{Direction, Ranger},
		store::MockKVStore,
	};

	fn setup_map() -> Result<
		(
			Map<Vec<u8>, Vec<u8>, BytesKeyCodec, BytesValueCodec>,
			SchemaBuilder<MockKVStore>,
		),
		CollectionError,
	> {
		let store = MockKVStore::new();
		let mut schema_builder = SchemaBuilder::new(store.clone());

		let map = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store)),
			b"test_prefix".to_vec(),
			"test_map".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)?;

		Ok((map, schema_builder))
	}

	#[test]
	fn test_basic_operations() {
		let result = setup_map();
		assert!(result.is_ok());
		let (map, _) = result.unwrap();

		let key = b"test_key".to_vec();
		let value = b"test_value".to_vec();

		// Test set
		map.set(&key, &value);

		// Test get
		let retrieved = map.get(&key);

		assert!(retrieved.is_ok());
		assert!(retrieved.unwrap() == value);

		// Test has
		let exists = map.has(&key);

		assert!(exists.is_ok());
		assert!(exists.unwrap(), "Key should exist after setting");

		// Test remove
		let removed = map.remove(&key);

		assert!(removed.is_ok());

		let retrieved_after_remove = map.get(&key);

		assert!(retrieved_after_remove.is_err());
		assert!(matches!(
			retrieved_after_remove.unwrap_err(),
			CollectionError::NotFoundError
		));
	}

	#[test]
	fn test_prefix_isolation() -> Result<(), CollectionError> {
		let store = MockKVStore::new();
		let mut schema_builder = SchemaBuilder::new(store.clone());

		// Create two maps with different prefixes
		let map1 = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store.clone())),
			b"prefix1".to_vec(),
			"map1".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)?;

		let map2 = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store)),
			b"prefix2".to_vec(),
			"map2".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)?;

		let key = b"same_key".to_vec();
		let value1 = b"value1".to_vec();
		let value2 = b"value2".to_vec();

		// Set values in both maps
		map1.set(&key, &value1)?;
		map2.set(&key, &value2)?;

		// Verify values are isolated
		assert_eq!(map1.get(&key)?, value1);
		assert_eq!(map2.get(&key)?, value2);

		Ok(())
	}

	#[test]
	fn test_same_prefix() {
		let store = MockKVStore::new();
		let mut schema_builder = SchemaBuilder::new(store.clone());

		// Create two maps with different prefixes
		let _ = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store.clone())),
			b"prefix1".to_vec(),
			"map1".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		);

		let _ = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store)),
			b"prefix11".to_vec(),
			"map2".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		);

		let schema = schema_builder.build();

		assert!(schema.is_err());
	}

	#[test]
	fn test_iterator() {
		let result = setup_map();
		assert!(result.is_ok());
		let (map, _) = result.unwrap();

		// Insert test data
		let test_data = vec![
			(b"a".to_vec(), b"1".to_vec()),
			(b"b".to_vec(), b"2".to_vec()),
			(b"c".to_vec(), b"3".to_vec()),
		];

		for (k, v) in &test_data {
			map.set(k, v);
			assert!(map.get(k).is_ok());
		}

		// Test ascending iteration
		let ranger = Ranger {
			start: Some(b"a".to_vec()),
			end: Some(b"c".to_vec()),
			direction: Direction::Asc,
		};

		let result = map.iter(ranger);
		assert!(result.is_ok());
		let iter = result.unwrap();

		let items: Vec<_> = iter.collect();

		assert_eq!(items.len(), 2); // Should include 'a' and 'b' but not 'c'
		assert_eq!(
			items[0].0,
			encode_key_with_prefix(&map.prefix, &test_data[0].0, &map.key_codec).unwrap()
		);
		assert_eq!(items[0].1, map.value_codec.encode(&test_data[0].1).unwrap());
		assert_eq!(
			items[1].0,
			encode_key_with_prefix(&map.prefix, &test_data[1].0, &map.key_codec).unwrap()
		);
		assert_eq!(items[1].1, map.value_codec.encode(&test_data[1].1).unwrap());

		// Test descending iteration
		let ranger = Ranger {
			start: Some(b"a".to_vec()),
			end: Some(b"c".to_vec()),
			direction: Direction::Desc,
		};

		let result = map.iter(ranger);
		assert!(result.is_ok());
		let iter = result.unwrap();

		let items: Vec<_> = iter.collect();

		assert_eq!(items.len(), 2);
		assert_eq!(
			items[0].0,
			encode_key_with_prefix(&map.prefix, &test_data[1].0, &map.key_codec).unwrap(),
		);
		assert_eq!(items[0].1, map.value_codec.encode(&test_data[1].1).unwrap(),);
		assert_eq!(
			items[1].0,
			encode_key_with_prefix(&map.prefix, &test_data[0].0, &map.key_codec).unwrap(),
		);
		assert_eq!(items[1].1, map.value_codec.encode(&test_data[0].1).unwrap(),);
	}

	#[test]
	fn test_schema_registration() {
		let store = MockKVStore::new();
		let mut schema_builder = SchemaBuilder::new(store.clone());

		// Create first map
		Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store.clone())),
			b"prefix1".to_vec(),
			"map1".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)
		.expect("First map creation should succeed");

		// Try to create map with same name
		let result = Map::new(
			&mut schema_builder,
			Arc::new(Box::new(store)),
			b"prefix2".to_vec(),
			"map1".to_string(), // Same name
			BytesKeyCodec,
			BytesValueCodec,
		);

		assert!(
			result.is_err(),
			"Creating map with duplicate name should fail"
		);
	}

	#[test]
	fn test_invalid_operations() -> Result<(), CollectionError> {
		let (map, _) = setup_map()?;

		// Test get with non-existent key
		let result = map.get(&b"nonexistent".to_vec());
		assert!(matches!(
			result.unwrap_err(),
			CollectionError::NotFoundError
		));

		// Test remove non-existent key (should not error)
		let result = map.remove(&b"nonexistent".to_vec());
		assert!(result.is_ok());

		Ok(())
	}
}
