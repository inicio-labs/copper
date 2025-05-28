use std::fmt::{self, Display};

use crate::util::{Address, TxHash};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidPoolTransactionError {
	ExceedsGasLimit(u128, u128),
	Overdraft { cost: u128, balance: u128 },
}

impl Display for InvalidPoolTransactionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ExceedsGasLimit(limit, actual) => {
				write!(f, "gas limit exceeded: {} > {}", actual, limit)
			},
			Self::Overdraft { cost, balance } => write!(f, "overdraft: {} > {}", cost, balance),
		}
	}
}
/// Transaction pool error.
#[derive(Debug, Error)]
#[error("[{hash}]: {kind}")]
pub struct PoolError {
	/// The transaction hash that caused the error.
	pub hash: TxHash,
	/// The error kind.
	pub kind: PoolErrorKind,
}

/// Transaction pool error kind.
#[derive(Debug, Error)]
pub enum PoolErrorKind {
	/// Same transaction already imported
	#[error("already imported")]
	AlreadyImported,
	/// Thrown if a replacement transaction's gas price is below the already imported transaction
	#[error("insufficient gas price to replace existing transaction")]
	ReplacementUnderpriced,
	/// The fee cap of the transaction is below the minimum fee cap determined by the protocol
	#[error("transaction feeCap {0} below chain minimum")]
	FeeCapBelowMinimumProtocolFeeCap(u128),
	/// Thrown when the number of unique transactions of a sender exceeded the slot capacity.
	#[error("rejected due to {0} being identified as a spammer")]
	SpammerExceededCapacity(Address),
	/// Thrown when a new transaction is added to the pool, but then immediately discarded to
	/// respect the size limits of the pool.
	#[error("transaction discarded outright due to pool size constraints")]
	DiscardedOnInsert,
	/// Thrown when the transaction is considered invalid.
	#[error(transparent)]
	InvalidTransaction(#[from] InvalidPoolTransactionError),

	/// error
	#[error(transparent)]
	Other(#[from] Box<dyn core::error::Error + Send + Sync>),
}

// === impl PoolError ===

impl PoolError {
	/// Creates a new pool error.
	pub fn new(hash: TxHash, kind: impl Into<PoolErrorKind>) -> Self {
		Self { hash, kind: kind.into() }
	}

	/// Creates a new pool error with the `Other` kind.
	pub fn other(
		hash: TxHash,
		error: impl Into<Box<dyn core::error::Error + Send + Sync>>,
	) -> Self {
		Self { hash, kind: PoolErrorKind::Other(error.into()) }
	}

	/// Returns `true` if the error was caused by a transaction that is considered bad in the
	/// context of the transaction pool and warrants peer penalization.
	///
	/// Not all error variants are caused by the incorrect composition of the transaction (See also
	/// [`InvalidPoolTransactionError`]) and can be caused by the current state of the transaction
	/// pool. For example the transaction pool is already full or the error was caused my an
	/// internal error, such as database errors.
	///
	/// This function returns true only if the transaction will never make it into the pool because
	/// its composition is invalid and the original sender should have detected this as well. This
	/// is used to determine whether the original sender should be penalized for sending an
	/// erroneous transaction.
	#[inline]
	pub fn is_bad_transaction(&self) -> bool {
		#[expect(clippy::match_same_arms)]
		match &self.kind {
			PoolErrorKind::AlreadyImported => {
				// already imported but not bad
				false
			},
			PoolErrorKind::ReplacementUnderpriced => {
				// already imported but not bad
				false
			},
			PoolErrorKind::FeeCapBelowMinimumProtocolFeeCap(_) => {
				// fee cap of the tx below the technical minimum determined by the protocol, see
				// [MINIMUM_PROTOCOL_FEE_CAP](alloy_primitives::constants::MIN_PROTOCOL_BASE_FEE)
				// although this transaction will always be invalid, we do not want to penalize the
				// sender because this check simply could not be implemented by the client
				false
			},
			PoolErrorKind::SpammerExceededCapacity(_) => {
				// the sender exceeded the slot capacity, we should not penalize the peer for
				// sending the tx because we don't know if all the transactions are sent from the
				// same peer, there's also a chance that old transactions haven't been cleared yet
				// (pool lags behind) and old transaction still occupy a slot in the pool
				false
			},
			PoolErrorKind::DiscardedOnInsert => {
				// valid tx but dropped due to size constraints
				false
			},
			PoolErrorKind::InvalidTransaction(_) => {
				// TODO: check if the transaction is bad
				// transaction rejected because it violates constraints
				false
			},
			PoolErrorKind::Other(_) => {
				// internal error unrelated to the transaction
				false
			},
		}
	}
}
