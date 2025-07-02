use crate::gas_store::{error::GasError, gas_store::Gas};

// GasConfig struct definition
#[derive(Debug, Clone)]
pub struct GasConfig {
	pub has_cost: Gas,
	pub delete_cost: Gas,
	pub read_cost_flat: Gas,
	pub read_cost_per_byte: Gas,
	pub write_cost_flat: Gas,
	pub write_cost_per_byte: Gas,
	pub iter_next_cost_flat: Gas,
}

impl GasConfig {
	/// Returns a default gas config for KVStores
	pub fn kv_gas_config() -> Self {
		Self {
			has_cost: 1000,
			delete_cost: 1000,
			read_cost_flat: 1000,
			read_cost_per_byte: 3,
			write_cost_flat: 2000,
			write_cost_per_byte: 30,
			iter_next_cost_flat: 30,
		}
	}

	/// Returns a default gas config for TransientStores
	pub fn transient_gas_config() -> Self {
		Self {
			has_cost: 100,
			delete_cost: 100,
			read_cost_flat: 100,
			read_cost_per_byte: 0,
			write_cost_flat: 200,
			write_cost_per_byte: 3,
			iter_next_cost_flat: 3,
		}
	}
}

// GasMeter trait definition
pub trait GasMeter {
	/// Returns the amount of gas consumed by the gas meter instance
	fn gas_consumed(&self) -> Gas;

	/// Returns the amount of gas consumed by gas meter instance, or the limit if it is reached
	fn gas_consumed_to_limit(&self) -> Gas;

	/// Returns the gas left in the GasMeter
	fn gas_remaining(&self) -> Gas;

	/// Returns the limit of the gas meter instance. 0 if the gas meter is infinite
	fn limit(&self) -> Gas;

	/// Consumes the amount of gas provided. Panics with descriptor if gas overflows
	fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasError>;

	/// Deducts the given amount from the gas consumed
	fn refund_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasError>;

	/// Returns true if the amount of gas consumed is strictly above the limit
	fn is_past_limit(&self) -> bool;

	/// Returns true if the amount of gas consumed is above or equal to the limit
	fn is_out_of_gas(&self) -> bool;

	/// String representation of the gas meter
	fn to_string(&self) -> String;
}

// Basic GasMeter implementation
#[derive(Debug)]
pub struct BasicGasMeter {
	limit: Gas,
	consumed: Gas,
}

impl BasicGasMeter {
	pub fn new(limit: Gas) -> Self {
		Self { limit, consumed: 0 }
	}

	fn add_uint64_overflow(a: u64, b: u64) -> (u64, bool) {
		match a.checked_add(b) {
			Some(sum) => (sum, false),
			None => (0, true),
		}
	}
}

impl GasMeter for BasicGasMeter {
	fn gas_consumed(&self) -> Gas {
		self.consumed
	}

	fn gas_consumed_to_limit(&self) -> Gas {
		if self.is_past_limit() {
			self.limit
		} else {
			self.consumed
		}
	}

	fn gas_remaining(&self) -> Gas {
		if self.is_past_limit() {
			0
		} else {
			self.limit - self.consumed
		}
	}

	fn limit(&self) -> Gas {
		self.limit
	}

	fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasError> {
		let (new_consumed, overflow) = Self::add_uint64_overflow(self.consumed, amount);
		if overflow {
			self.consumed = u64::MAX;
			return Err(GasError::GasOverflow { descriptor: descriptor.to_string() });
		}

		self.consumed = new_consumed;
		if self.consumed > self.limit {
			return Err(GasError::OutOfGas { descriptor: descriptor.to_string() });
		}

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
		self.consumed > self.limit
	}

	fn is_out_of_gas(&self) -> bool {
		self.consumed >= self.limit
	}

	fn to_string(&self) -> String {
		format!(
			"BasicGasMeter:\n  limit: {}\n  consumed: {}",
			self.limit, self.consumed
		)
	}
}
