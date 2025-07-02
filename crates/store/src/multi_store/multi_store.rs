mod constants;
mod db;
mod error;
pub mod store;

pub use self::db::{BatchWriter, Database};
pub use self::error::StoreError;

use constants::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::types::committer::Committer;

use crate::iavl::iavl_store::IAVLStore;
use crate::iavl::tree::Tree;
use crate::multi_store::multi_store::store::Store;
use crate::types::kv_store::KVStore;
use crate::types::query::Queryable;
use crate::types::query::{RequestQuery, ResponseQuery};
use crate::types::store::{Store as StoreTrait, StoreInfo, StoreType, StoreWithInitialVersion};
use crate::types::{
	commit_info::CommitInfo, commit_kv_store::CommitKVStore, committer::CommitID, header::Header,
	store::StoreParams, store_key::StoreKey,
};
use sha2::{Digest, Sha256};

use self::error::Result;

/// Store is composed of many CommitStores. It implements the CommitMultiStore interface.
/// This is the main multistore that manages multiple substores.
pub struct MultiStore<DB: Database, PruningManager, SK: StoreKey> {
	db: DB,
	last_commit_info: Option<CommitInfo>,
	pruning_manager: PruningManager,
	iavl_cache_size: i32,
	iavl_disable_fast_node: bool,
	stores_params: HashMap<SK, StoreParams<DB, SK>>,
	stores: HashMap<SK, Arc<RwLock<Store>>>,
	keys_by_name: HashMap<String, SK>,
	initial_version: i64,
	removal_map: HashMap<SK, bool>,

	commit_header: Header,
}

impl<DB, PruningManager, SK: StoreKey> MultiStore<DB, PruningManager, SK>
where
	DB: Database + Clone,
	PruningManager: Default,
{
	// Constructor and Basic Configuration

	/// Creates a new Store instance with the provided database, logger, and metrics
	pub fn new(db: DB) -> Self {
		Self {
			db,
			last_commit_info: None,
			pruning_manager: PruningManager::default(),
			iavl_cache_size: DEFAULT_IAVL_CACHE_SIZE,
			iavl_disable_fast_node: DEFAULT_IAVL_DISABLE_FAST_NODE,
			stores_params: HashMap::new(),
			stores: HashMap::new(),
			keys_by_name: HashMap::new(),
			initial_version: 0,
			removal_map: HashMap::new(),
			commit_header: Header::default(),
		}
	}

	/// Sets the cache size for IAVL tree operations
	pub fn set_iavl_cache_size(&mut self, cache_size: i32) {
		self.iavl_cache_size = cache_size;
	}

	/// Enables or disables the fast node feature on IAVL trees
	pub fn set_iavl_disable_fast_node(&mut self, disable: bool) {
		self.iavl_disable_fast_node = disable;
	}

	/// Returns the store type as StoreTypeMulti
	pub fn get_store_type(&self) -> StoreType {
		StoreType::MultiStore
	}

	// Store Management

	/// Mounts a store with the given StoreKey, StoreType, and database
	pub fn mount_store_with_db(
		&mut self,
		key: SK,
		store_type: StoreType,
		db: DB,
	) -> Result<(), StoreError> {
		let key_name = key.name().to_string();

		if self.stores_params.contains_key(&key) {
			return Err(StoreError::DuplicateKey(key.to_string()));
		}

		if self.keys_by_name.contains_key(&key_name) {
			return Err(StoreError::DuplicateKeyName(key_name));
		}

		let params = StoreParams::new(db, key.clone(), store_type, 0);
		self.stores_params.insert(key.clone(), params);
		self.keys_by_name.insert(key_name, key.clone());

		Ok(())
	}

	/// Returns a mounted CommitStore for a given StoreKey
	pub fn get_commit_store(&self, key: SK) -> Result<&Arc<RwLock<Store>>> {
		self.get_commit_kv_store(key)
	}

	/// Returns a mounted CommitKVStore for a given StoreKey
	pub fn get_commit_kv_store(&self, key: SK) -> Result<&Arc<RwLock<Store>>> {
		self.stores
			.get(&key)
			.map(|store| store)
			.ok_or_else(|| StoreError::StoreNotFound(key.to_string()))
	}

	/// Returns mapping of store names to StoreKeys
	pub fn store_keys_by_name(&self) -> &HashMap<String, SK> {
		&self.keys_by_name
	}

	// Version Loading

	/// Loads the latest persisted version of the store
	pub fn load_latest_version(&mut self) -> Result<()> {
		let version = Self::get_latest_version(&self.db)?;
		self.load_version(version)
	}

	/// Loads a specific persisted version of the store
	pub fn load_version(&mut self, version: i64) -> Result<()> {
		self.load_version_internal(version)
	}

	/// Internal method that loads a specific version of the store
	fn load_version_internal(&mut self, version: i64) -> Result<()> {
		let mut infos = HashMap::new();
		let commit_info = if version != 0 {
			let info = self.get_commit_info(version)?;
			for store_info in &info.store_infos {
				infos.insert(store_info.name.clone(), store_info.clone());
			}
			Some(info)
		} else {
			Some(CommitInfo::new(0, vec![], SystemTime::now()))
		};

		let mut new_stores = HashMap::new();
		let store_keys: Vec<_> = self.stores_params.keys().cloned().collect();

		for key in store_keys {
			let params = self.stores_params[&key].clone();
			let commit_id = self.get_commit_id(&infos, &key.name());

			if commit_id.version != version && params.store_type == StoreType::IAVL {
				return Err(StoreError::VersionMismatch {
					expected: version,
					actual: commit_id.version,
				});
			}

			let store = self.load_commit_store_from_params(key.name(), commit_id, params)?;
			new_stores.insert(key, store);
		}

		self.last_commit_info = commit_info;
		self.stores = new_stores;

		Ok(())
	}

	// State and Versioning

	/// Returns the latest version number stored in the multi-store
	pub fn latest_version(&self) -> i64 {
		self.last_commit_id().version
	}

	/// Returns the last commit ID
	pub fn last_commit_id(&self) -> CommitID {
		match &self.last_commit_info {
			None => {
				let empty_hash = Sha256::digest(&[]).to_vec();
				CommitID {
					version: Self::get_latest_version(&self.db).unwrap_or(0),
					hash: empty_hash,
				}
			},
			Some(info) => {
				let mut commit_id = info.commit_id();
				if commit_id.hash.is_empty() {
					commit_id.hash = Sha256::digest(&[]).to_vec();
				}
				commit_id
			},
		}
	}

	/// Commits all stores and returns a new CommitID
	pub fn commit(&mut self) -> Result<CommitID> {
		let previous_height = self.last_commit_info.as_ref().map(|info| info.version).unwrap_or(0);

		let version = if previous_height == 0 && self.initial_version > 1 {
			self.initial_version
		} else {
			previous_height + 1
		};

		if self.commit_header.height != version {
			panic!(
				"commit header and version mismatch header_height={} version={}",
				self.commit_header.height, version
			);
		}

		self.last_commit_info = Some(Self::commit_stores(
			version,
			&mut self.stores,
			&self.removal_map,
		)?);

		self.last_commit_info.as_mut().unwrap().timestamp = self.commit_header.time;

		if let Some(ref mut info) = self.last_commit_info {
			info.timestamp = self.commit_header.time;
		}

		self.flush_metadata(version, self.last_commit_info.as_ref())?;

		// Remove remnants of removed stores
		let keys_to_remove: Vec<_> = self
			.removal_map
			.iter()
			.filter(|(_, should_remove)| **should_remove)
			.map(|(key, _)| key.clone())
			.collect();

		for key in keys_to_remove {
			self.stores.remove(&key);
			self.stores_params.remove(&key);
			self.keys_by_name.remove(&key.name().to_string());
		}

		// Reset removal map
		self.removal_map.clear();

		Ok(CommitID { version, hash: self.last_commit_info.as_ref().unwrap().hash() })
	}

	/// Returns the current working hash of all IAVL stores before commit
	pub fn working_hash(&self) -> Result<Vec<u8>> {
		let mut store_infos = Vec::new();
		let mut store_keys: Vec<_> = self.stores.keys().collect();
		store_keys.sort_by(|a, b| a.name().cmp(b.name()));

		for key in store_keys {
			let store = &self.stores[key];

			if store.read().unwrap().get_store_type() != StoreType::IAVL {
				continue;
			}

			if !self.removal_map.get(key).unwrap_or(&false) {
				let store_info = StoreInfo {
					name: key.name().to_string(),
					// TODO: check this
					commit_id: CommitID { version: 0, hash: store.read()?.working_hash() },
				};
				store_infos.push(store_info);
			}
		}

		store_infos.sort_by(|a, b| a.name.cmp(&b.name));
		Ok(CommitInfo { version: 0, store_infos, timestamp: std::time::SystemTime::now() }.hash())
	}

	// Store Access

	/// Returns a mounted Store for a given StoreKey
	pub fn get_store(&self, key: SK) -> Result<&Arc<RwLock<Store>>> {
		let store = self.get_store_by_name(key.name())?;
		Ok(store)
	}

	/// Returns a mounted KVStore for a given StoreKey
	pub fn get_kv_store(&self, key: SK) -> Result<&Arc<RwLock<impl KVStore>>> {
		let store = self.get_commit_kv_store(key)?;
		Ok(store)
	}

	/// Performs a lookup of a StoreKey by store name and returns the corresponding Store
	pub fn get_store_by_name(&self, name: &str) -> Result<&Arc<RwLock<Store>>> {
		let key = self.keys_by_name.get(name).cloned();
		if key.is_none() {
			return Err(StoreError::StoreNotFound(name.to_string()));
		}

		self.get_store(key.unwrap())
	}

	// Querying

	/// Handles queries by parsing the path and delegating to the appropriate store
	pub fn query(&self, req: &RequestQuery) -> Result<ResponseQuery> {
		let (store_name, subpath) = Self::parse_path(&req.path)?;

		let store = self.get_store_by_name(&store_name)?;

		let res = store.read()?.query(req);

		let mut modified_req = req.clone();
		modified_req.path = subpath;

		// TODO: handle proof
		res
	}

	/// Utility function that parses a path and extracts store name and subpath
	fn parse_path(path: &str) -> Result<(String, String)> {
		if !path.starts_with('/') {
			return Err(StoreError::InvalidPath(path.to_string()));
		}

		let parts: Vec<&str> = path[1..].splitn(2, '/').collect();
		let store_name = parts[0].to_string();
		let subpath = if parts.len() == 2 {
			format!("/{}", parts[1])
		} else {
			String::new()
		};

		Ok((store_name, subpath))
	}

	// Initial Version Management

	/// Sets the initial version of the IAVL tree for new chains
	pub fn set_initial_version(&mut self, version: i64) -> Result<(), StoreError> {
		self.initial_version = version;

		for (_, store) in self.stores.iter() {
			if store.read()?.get_store_type() == StoreType::IAVL {
				store.write()?.set_initial_version(version);
			}
		}

		Ok(())
	}

	// Utility Functions

	/// Retrieves the CommitID for a store by name
	fn get_commit_id(&self, infos: &HashMap<String, StoreInfo>, name: &str) -> CommitID {
		infos.get(name).map(|info| info.commit_id.clone()).unwrap_or_default()
	}

	/// Utility function that deletes all key-value pairs from a KVStore
	fn delete_kv_store(store: &Arc<RwLock<impl KVStore>>) -> Result<(), StoreError> {
		let mut iter = store.read()?.iterator(None, None);

		while let Some((key, value)) = iter.next() {
			store.write()?.delete(&key);
		}

		Ok(())
	}

	/// Utility function that moves all data from one KVStore to another
	fn move_kv_store_data(
		old_store: &Arc<RwLock<impl KVStore>>,
		new_store: &Arc<RwLock<impl KVStore>>,
	) -> Result<(), StoreError> {
		let mut iter = old_store.read()?.iterator(None, None);

		while let Some((key, value)) = iter.next() {
			new_store.write()?.set(&key, &value);
		}

		Self::delete_kv_store(old_store)
	}

	// Metadata Management

	/// Builds CommitInfo for a given version
	fn build_commit_info(&self, version: i64) -> CommitInfo {
		let mut store_infos = Vec::new();
		let mut store_keys: Vec<_> = self.stores.keys().collect();
		store_keys.sort_by(|a, b| a.name().cmp(b.name()));

		for key in store_keys {
			let store = &self.stores[key];
			let store_type = store.read().unwrap().get_store_type();

			if store_type == StoreType::Transient || store_type == StoreType::Memory {
				continue;
			}

			store_infos.push(StoreInfo::new(
				key.name().to_string(),
				store.read().unwrap().last_commit_id(),
			));
		}

		//TODO: Check this
		CommitInfo::new(version, store_infos, std::time::SystemTime::now())
	}

	/// Sets the commit block header for the store
	pub fn set_commit_header(&mut self, header: Header) {
		self.commit_header = header;
	}

	/// Retrieves CommitInfo for a specific version/height from the database
	pub fn get_commit_info(&self, version: i64) -> Result<CommitInfo, StoreError> {
		let key = format!("s/{}", version);
		let data = self
			.db
			.get(key.as_bytes())
			.map_err(|e| StoreError::DatabaseError(e.to_string()))?
			.ok_or_else(|| StoreError::DatabaseError("no commit info found".to_string()))?;

		CommitInfo::unmarshal(&data).map_err(|e| {
			StoreError::DatabaseError(format!("failed to unmarshal commit info: {}", e))
		})
	}

	/// Flushes commit metadata to the database
	fn flush_metadata(
		&self,
		version: i64,
		commit_info: Option<&CommitInfo>,
	) -> Result<(), StoreError> {
		let mut batch = self.db.new_batch();

		if let Some(info) = commit_info {
			Self::flush_commit_info(&mut batch, version, info)?;
		}
		Self::flush_latest_version(&mut batch, version)?;
		batch.write_sync().map_err(|e| StoreError::DatabaseError(e.to_string()))?;

		Ok(())
	}

	// Store Loading

	/// Internal method that loads a CommitKVStore from the provided parameters
	fn load_commit_store_from_params(
		&self,
		key_name: &str,
		commit_id: CommitID,
		params: StoreParams<DB, SK>,
	) -> Result<Arc<RwLock<Store>>>
	where
		DB: Database,
	{
		let db = params.db.with_prefix(b"s/_/");

		match params.store_type {
			StoreType::MultiStore => {
				return panic!("MultiStore is not supported");
			},

			StoreType::IAVL => {
				let store = IAVLStore::load_store_with_initial_version(
					db,
					params.key,
					commit_id,
					params.initial_version,
					self.iavl_cache_size,
					self.iavl_disable_fast_node,
				)
				.map_err(|e| StoreError::IAVLLoadError(e.to_string()))?;

				Ok(Arc::new(RwLock::new(Store::IAVL(store))))
			},
			_ => Err(StoreError::UnsupportedStoreType(params.store_type)),
		}
	}

	// Static Utility Functions

	/// Static function that retrieves the latest version number from the database
	pub fn get_latest_version(db: &DB) -> Result<i64> {
		let data = db.get(LATEST_KEY).map_err(|e| StoreError::DatabaseError(e.to_string()))?;

		match data {
			Some(bytes) => {
				let version = i64::from_be_bytes(bytes.try_into().map_err(|_| {
					StoreError::DatabaseError("invalid version format".to_string())
				})?);
				Ok(version)
			},
			None => Ok(0),
		}
	}

	/// Static function that commits all stores and returns CommitInfo
	fn commit_stores(
		version: i64,
		stores: &mut HashMap<SK, Arc<RwLock<impl CommitKVStore>>>,
		removal_map: &HashMap<SK, bool>,
	) -> Result<CommitInfo> {
		let mut store_infos = Vec::new();
		let mut store_keys: Vec<_> = stores.keys().cloned().collect();
		store_keys.sort_by(|a, b| a.name().cmp(b.name()));

		for key in store_keys {
			let store = stores.get_mut(&key).unwrap();
			let last = store.read()?.last_commit_id();

			let commit_id = if last.version >= version {
				CommitID { version, hash: last.hash }
			} else {
				store.write()?.commit()
			};

			let store_type = store.read()?.get_store_type();
			if store_type == StoreType::Transient || store_type == StoreType::Memory {
				continue;
			}

			if !removal_map.get(&key).unwrap_or(&false) {
				store_infos.push(StoreInfo::new(key.name().to_string(), commit_id));
			}
		}

		store_infos.sort_by(|a, b| a.name.cmp(&b.name));

		Ok(CommitInfo::new(
			version,
			store_infos,
			std::time::SystemTime::now(),
		))
	}

	/// Static function that marshals CommitInfo and writes it to database
	fn flush_commit_info(
		batch: &mut impl BatchWriter,
		version: i64,
		commit_info: &CommitInfo,
	) -> Result<()> {
		let data = commit_info.marshal().map_err(|e| StoreError::DatabaseError(e.to_string()))?;
		let key = format!("s/{}", version);
		batch.set(key.as_bytes(), &data).map_err(|e| StoreError::DatabaseError(e.to_string()))
	}

	/// Static function that marshals and writes the latest version number to database
	fn flush_latest_version(batch: &mut impl BatchWriter, version: i64) -> Result<()> {
		let data = version.to_be_bytes();
		batch.set(LATEST_KEY, &data).map_err(|e| StoreError::DatabaseError(e.to_string()))
	}
}
