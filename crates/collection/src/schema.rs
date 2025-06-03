use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{collection::Collection, store::KVStore, CollectionError};

/// Regular expression for valid collection names: must start with a letter and contain only alphanumeric characters or underscores
pub const NAME_REGEX: &str = r"^[a-zA-Z][a-zA-Z0-9_]*$";

lazy_static! {
	static ref NAME_VALIDATOR: Regex = Regex::new(NAME_REGEX).unwrap();
}

/// Schema represents a collection of collections that can be used to store and retrieve data.
#[derive(Default, Clone)]
pub struct Schema<T: KVStore<CollectionError> + Clone> {
	store_accessor: T,
	collections_ordered: Vec<String>,
	collections_by_prefix: HashMap<String, Arc<Box<dyn Collection>>>,
	collections_by_name: HashMap<String, Arc<Box<dyn Collection>>>,
	// Fields will be added as we implement more functionality
}

pub struct SchemaBuilder<T: KVStore<CollectionError> + Clone> {
	schema: Schema<T>,
	built: bool,
}

impl<T: KVStore<CollectionError> + Clone> SchemaBuilder<T> {
	pub fn new(store: T) -> Self {
		Self {
			schema: Schema {
				store_accessor: store,
				collections_ordered: Default::default(),
				collections_by_prefix: Default::default(),
				collections_by_name: Default::default(),
			},
			built: false,
		}
	}

	pub fn add_collection(
		&mut self,
		collection: Arc<Box<dyn Collection>>,
	) -> Result<(), CollectionError> {
		let prefix = collection.get_prefix();
		let name = collection.get_name();

		if self.schema.collections_by_prefix.contains_key(&hex::encode(prefix.clone())) {
			return Err(CollectionError::PrefixTakenError(hex::encode(
				prefix.clone(),
			)));
		}

		if self.schema.collections_by_name.contains_key(&name) {
			return Err(CollectionError::NameTakenError(name.clone()));
		}

		if !NAME_VALIDATOR.is_match(&name) {
			return Err(CollectionError::NameRegexError(name.clone()));
		}

		self.schema.collections_by_prefix.insert(hex::encode(prefix.clone()), collection.clone());
		self.schema.collections_by_name.insert(name.clone(), collection.clone());
		self.schema.collections_ordered.push(name);

		Ok(())
	}

	pub fn build(&mut self) -> Result<Schema<T>, CollectionError> {
		if self.built {
			return Err(CollectionError::EncodeError(
				"Schema already built".to_string(),
			));
		}

		for prefix in self.schema.collections_by_prefix.keys() {
			for prefix2 in self.schema.collections_by_prefix.keys() {
				if prefix == prefix2 {
					continue;
				}
				if prefix.starts_with(prefix2) {
					return Err(CollectionError::PrefixOverlapError(
						prefix.clone(),
						prefix2.clone(),
					));
				}
			}
		}

		let mut collection_ordered = vec![];

		for keys in self.schema.collections_by_name.keys() {
			collection_ordered.push(keys.to_string());
		}

		collection_ordered.sort();

		self.schema.collections_ordered = collection_ordered;

		self.built = true;

		Ok(self.schema.clone())
	}
}
