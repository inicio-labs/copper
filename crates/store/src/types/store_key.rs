use std::hash::{Hash, Hasher};

/// StoreKey is used to index stores in a MultiStore
pub trait StoreKey: Clone + Eq + Hash {
	/// Get the name of the store key
	fn name(&self) -> &str;

	/// String representation of the key
	fn to_string(&self) -> String;
}

/// KVStoreKey implementation - used for regular persistent stores
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KVStoreKey {
	name: String,
}

impl KVStoreKey {
	pub fn new(name: String) -> Self {
		if name.is_empty() {
			panic!("empty key name not allowed");
		}
		Self { name }
	}
}

impl StoreKey for KVStoreKey {
	fn name(&self) -> &str {
		&self.name
	}

	fn to_string(&self) -> String {
		format!("KVStoreKey{{{}}}", self.name)
	}
}

impl Hash for KVStoreKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl std::fmt::Display for KVStoreKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "KVStoreKey{{{}}}", self.name)
	}
}

/// TransientStoreKey - used for transient stores that are reset each block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransientStoreKey {
	name: String,
}

impl TransientStoreKey {
	pub fn new(name: String) -> Self {
		Self { name }
	}
}

impl Hash for TransientStoreKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl StoreKey for TransientStoreKey {
	fn name(&self) -> &str {
		&self.name
	}

	fn to_string(&self) -> String {
		format!("TransientStoreKey{{{}}}", self.name)
	}
}

/// MemoryStoreKey - used for in-memory stores
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStoreKey {
	name: String,
}

impl MemoryStoreKey {
	pub fn new(name: String) -> Self {
		Self { name }
	}
}

impl Hash for MemoryStoreKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl StoreKey for MemoryStoreKey {
	fn name(&self) -> &str {
		&self.name
	}

	fn to_string(&self) -> String {
		format!("MemoryStoreKey{{{}}}", self.name)
	}
}
