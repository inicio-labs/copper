use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::gas_store::gas::{GasConfig, GasMeter};
use crate::types::kv_store::KVStore;
use crate::types::store::{Store, StoreType};

// Gas type alias
pub type Gas = u64;

// Infinite GasMeter implementation
#[derive(Debug)]
pub struct InfiniteGasMeter {
	consumed: Gas,
}

impl InfiniteGasMeter {
	pub fn new() -> Self {
		Self { consumed: 0 }
	}
}

impl GasMeter for InfiniteGasMeter {
	fn gas_consumed(&self) -> Gas {
		self.consumed
	}

	fn gas_consumed_to_limit(&self) -> Gas {
		self.consumed
	}

	fn gas_remaining(&self) -> Gas {
		u64::MAX
	}

	fn limit(&self) -> Gas {
		u64::MAX
	}

	fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasError> {
		let (new_consumed, overflow) = BasicGasMeter::add_uint64_overflow(self.consumed, amount);
		if overflow {
			return Err(GasError::GasOverflow { descriptor: descriptor.to_string() });
		}

		self.consumed = new_consumed;
		Ok(())
	}

	fn refund_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasError> {
		if self.consumed < amount {
			return Err(GasError::NegativeGasConsumed { descriptor: descriptor.to_string() });
		}

		self.consumed -= amount;
		Ok(())
	}

	fn is_past_limit(&self) -> bool {
		false
	}

	fn is_out_of_gas(&self) -> bool {
		false
	}

	fn to_string(&self) -> String {
		format!("InfiniteGasMeter:\n  consumed: {}", self.consumed)
	}
}

// GasKVStore - KVStore wrapper that tracks gas consumption
#[derive(Debug)]
pub struct GasKVStore<T: KVStore, G: GasMeter> {
	parent: T,
	gas_meter: Arc<Mutex<G>>,
	gas_config: GasConfig,
}

impl<T: KVStore, G: GasMeter> GasKVStore<T, G> {
	pub fn new(parent: T, gas_meter: Arc<Mutex<G>>, gas_config: GasConfig) -> Self {
		Self { parent, gas_meter, gas_config }
	}
}

impl<T: KVStore, G: GasMeter> Store for GasKVStore<T, G> {
	fn get_store_type(&self) -> StoreType {
		self.parent.get_store_type()
	}
}

impl<T: KVStore, G: GasMeter> KVStore for GasKVStore<T, G> {
	/// Get value by key, returns None if key doesn't exist
	fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.gas_meter.lock().unwrap().consume_gas(1, "get")?;
		self.parent.get(key)
	}

	/// Check if key exists
	fn has(&self, key: &[u8]) -> bool;

	/// Set key-value pair
	fn set(&mut self, key: &[u8], value: &[u8]);

	/// Delete key
	fn delete(&mut self, key: &[u8]);

	/// Create iterator over key range (start inclusive, end exclusive)
	fn iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;

	// TODO: Change the Iterator struct later.
	/// Create reverse iterator over key range
	fn reverse_iterator(
		&self,
		start: Option<&[u8]>,
		end: Option<&[u8]>,
	) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;
}
