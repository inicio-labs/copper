use crate::multi_store::multi_store::StoreError;

pub type Result<T, E = StoreError> = core::result::Result<T, E>;

// Error types for gas operations
#[derive(Debug, Clone)]
pub enum GasError {
	OutOfGas { descriptor: String },
	GasOverflow { descriptor: String },
	NegativeGasConsumed { descriptor: String },
}

impl std::fmt::Display for GasError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GasError::OutOfGas { descriptor } => write!(f, "out of gas: {}", descriptor),
			GasError::GasOverflow { descriptor } => write!(f, "gas overflow: {}", descriptor),
			GasError::NegativeGasConsumed { descriptor } => {
				write!(f, "negative gas consumed: {}", descriptor)
			},
		}
	}
}
