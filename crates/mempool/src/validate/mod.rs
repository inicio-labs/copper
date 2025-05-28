//! Transaction validation abstractions.

use crate::{
	error::InvalidPoolTransactionError,
	identifier::{SenderId, TransactionId},
	traits::{PoolTransaction, TransactionOrigin},
	util::{Address, Hash, TxHash},
};

use futures_util::future::Either;
use std::{fmt, fmt::Debug, future::Future, time::Instant};

mod constants;

/// A Result type returned after checking a transaction's validity.
#[derive(Debug)]
pub enum TransactionValidationOutcome<T: PoolTransaction> {
	/// The transaction is considered _currently_ valid and can be inserted into the pool.
	Valid {
		transaction: T,
		/// Balance of the sender at the current point.
		balance: u128,
		/// Current nonce of the sender.
		state_nonce: u64,
		/// Whether to propagate the transaction to the network.
		propagate: bool,
	},
	/// The transaction is considered invalid indefinitely: It violates constraints that prevent
	/// this transaction from ever becoming valid.
	Invalid(T, InvalidPoolTransactionError),
	/// An error occurred while trying to validate the transaction
	Error(TxHash, Box<dyn core::error::Error + Send + Sync>),
}

impl<T: PoolTransaction> TransactionValidationOutcome<T> {
	/// Returns the hash of the transactions
	pub fn tx_hash(&self) -> TxHash {
		match self {
			Self::Valid { transaction, .. } => *transaction.hash(),
			Self::Invalid(transaction, ..) => *transaction.hash(),
			Self::Error(hash, ..) => *hash,
		}
	}

	/// Returns true if the transaction is valid.
	pub const fn is_valid(&self) -> bool {
		matches!(self, Self::Valid { .. })
	}

	/// Returns true if the transaction is invalid.
	pub const fn is_invalid(&self) -> bool {
		matches!(self, Self::Invalid(_, _))
	}

	/// Returns true if validation resulted in an error.
	pub const fn is_error(&self) -> bool {
		matches!(self, Self::Error(_, _))
	}
}

/// A wrapper type for a transaction that is valid and has an optional extracted EIP-4844 blob
/// transaction sidecar.
///
/// If this is provided, then the sidecar will be temporarily stored in the blob store until the
/// transaction is finalized.
///
/// Note: Since blob transactions can be re-injected without their sidecar (after reorg), the
/// validator can omit the sidecar if it is still in the blob store and return a
/// [`ValidTransaction::Valid`] instead.
#[derive(Debug)]
pub enum ValidTransaction<T> {
	/// A valid transaction.
	Valid(T),
}

impl<T> ValidTransaction<T> {
	/// Creates a new valid transaction with an optional sidecar.
	pub fn new(transaction: T) -> Self {
		Self::Valid(transaction)
	}
}

impl<T: PoolTransaction> ValidTransaction<T> {
	/// Returns the transaction.
	#[inline]
	pub const fn transaction(&self) -> &T {
		match self {
			Self::Valid(transaction) => transaction,
		}
	}

	/// Consumes the wrapper and returns the transaction.
	pub fn into_transaction(self) -> T {
		match self {
			Self::Valid(transaction) => transaction,
		}
	}

	/// Returns the address of that transaction.
	#[inline]
	pub(crate) fn sender(&self) -> Address {
		self.transaction().sender()
	}

	/// Returns the hash of the transaction.
	#[inline]
	pub fn hash(&self) -> &TxHash {
		self.transaction().hash()
	}

	/// Returns the nonce of the transaction.
	#[inline]
	pub fn nonce(&self) -> u64 {
		self.transaction().nonce()
	}
}

/// Provides support for validating transaction at any given state of the chain
pub trait TransactionValidator: Debug + Send + Sync {
	/// The transaction type to validate.
	type Transaction: PoolTransaction;

	/// Validates the transaction and returns a [`TransactionValidationOutcome`] describing the
	/// validity of the given transaction.
	///
	/// This will be used by the transaction-pool to check whether the transaction should be
	/// inserted into the pool or discarded right away.
	///
	/// Implementers of this trait must ensure that the transaction is well-formed, i.e. that it
	/// complies at least all static constraints, which includes checking for:
	///
	///    * chain id
	///    * gas limit
	///    * max cost
	///    * nonce >= next nonce of the sender
	///    * ...
	///
	/// See [`InvalidTransactionError`](reth_primitives_traits::transaction::error::InvalidTransactionError) for common
	/// errors variants.
	///
	/// The transaction pool makes no additional assumptions about the validity of the transaction
	/// at the time of this call before it inserts it into the pool. However, the validity of
	/// this transaction is still subject to future (dynamic) changes enforced by the pool, for
	/// example nonce or balance changes. Hence, any validation checks must be applied in this
	/// function.
	///
	/// See [`TransactionValidationTaskExecutor`] for a reference implementation.
	fn validate_transaction(
		&self,
		origin: TransactionOrigin,
		transaction: Self::Transaction,
	) -> impl Future<Output = TransactionValidationOutcome<Self::Transaction>> + Send;

	/// Validates a batch of transactions.
	///
	/// Must return all outcomes for the given transactions in the same order.
	///
	/// See also [`Self::validate_transaction`].
	fn validate_transactions(
		&self,
		transactions: Vec<(TransactionOrigin, Self::Transaction)>,
	) -> impl Future<Output = Vec<TransactionValidationOutcome<Self::Transaction>>> + Send {
		async {
			futures_util::future::join_all(
				transactions.into_iter().map(|(origin, tx)| self.validate_transaction(origin, tx)),
			)
			.await
		}
	}

	// /// Invoked when the head block changes.
	// ///
	// /// This can be used to update fork specific values (timestamp).
	// fn on_new_head_block<B>(&self, _new_tip_block: &SealedBlock<B>)
	// where
	// 	B: Block,
	// {
	// }
}

impl<A, B> TransactionValidator for Either<A, B>
where
	A: TransactionValidator,
	B: TransactionValidator<Transaction = A::Transaction>,
{
	type Transaction = A::Transaction;

	async fn validate_transaction(
		&self,
		origin: TransactionOrigin,
		transaction: Self::Transaction,
	) -> TransactionValidationOutcome<Self::Transaction> {
		match self {
			Self::Left(v) => v.validate_transaction(origin, transaction).await,
			Self::Right(v) => v.validate_transaction(origin, transaction).await,
		}
	}

	async fn validate_transactions(
		&self,
		transactions: Vec<(TransactionOrigin, Self::Transaction)>,
	) -> Vec<TransactionValidationOutcome<Self::Transaction>> {
		match self {
			Self::Left(v) => v.validate_transactions(transactions).await,
			Self::Right(v) => v.validate_transactions(transactions).await,
		}
	}

	// fn on_new_head_block<Bl>(&self, new_tip_block: &SealedBlock<Bl>)
	// where
	// 	Bl: Block,
	// {
	// 	match self {
	// 		Self::Left(v) => v.on_new_head_block(new_tip_block),
	// 		Self::Right(v) => v.on_new_head_block(new_tip_block),
	// 	}
	// }
}

/// A valid transaction in the pool.
///
/// This is used as the internal representation of a transaction inside the pool.
///
/// For EIP-4844 blob transactions this will _not_ contain the blob sidecar which is stored
/// separately in the [`BlobStore`](crate::blobstore::BlobStore).
pub struct ValidPoolTransaction<T: PoolTransaction> {
	/// The transaction
	pub transaction: T,
	/// The identifier for this transaction.
	pub transaction_id: TransactionId,
	/// Whether it is allowed to propagate the transaction.
	pub propagate: bool,
	/// Timestamp when this was added to the pool.
	pub timestamp: Instant,
	/// Where this transaction originated from.
	pub origin: TransactionOrigin,
}

// === impl ValidPoolTransaction ===

impl<T: PoolTransaction> ValidPoolTransaction<T> {
	/// Returns the hash of the transaction.
	pub fn hash(&self) -> &TxHash {
		self.transaction.hash()
	}

	/// Returns the address of the sender
	pub fn sender(&self) -> Address {
		self.transaction.sender()
	}

	/// Returns a reference to the address of the sender
	pub fn sender_ref(&self) -> &Address {
		self.transaction.sender_ref()
	}

	/// Returns the internal identifier for the sender of this transaction
	pub(crate) const fn sender_id(&self) -> SenderId {
		self.transaction_id.sender
	}

	/// Returns the internal identifier for this transaction.
	pub(crate) const fn id(&self) -> &TransactionId {
		&self.transaction_id
	}

	/// Returns the length of the rlp encoded transaction
	#[inline]
	pub fn encoded_length(&self) -> usize {
		self.transaction.encoded_length()
	}

	/// Returns the nonce set for this transaction.
	pub fn nonce(&self) -> u64 {
		self.transaction.nonce()
	}

	/// Returns the cost that this transaction is allowed to consume:
	///
	/// For EIP-1559 transactions: `max_fee_per_gas * gas_limit + tx_value`.
	/// For legacy transactions: `gas_price * gas_limit + tx_value`.
	pub fn cost(&self) -> u128 {
		self.transaction.cost()
	}

	/// Returns the EIP-1559 Max base fee the caller is willing to pay.
	///
	/// For legacy transactions this is `gas_price`.
	pub fn max_fee_per_gas(&self) -> u128 {
		self.transaction.max_fee_per_gas()
	}

	/// Maximum amount of gas that the transaction is allowed to consume.
	pub fn gas_limit(&self) -> u128 {
		self.transaction.gas()
	}

	/// Whether the transaction originated locally.
	pub const fn is_local(&self) -> bool {
		self.origin.is_local()
	}

	/// The heap allocated size of this transaction.
	pub(crate) fn size(&self) -> usize {
		self.transaction.size()
	}
	/// an existing transaction in the pool.
	///
	/// A transaction is considered underpriced if it doesn't meet the required fee bump threshold.
	/// This applies to both standard gas fees and, for blob-carrying transactions (EIP-4844),
	/// the blob-specific fees.
	#[inline]
	pub(crate) fn is_underpriced(&self, maybe_replacement: &Self) -> bool {
		// Check if the max fee per gas is underpriced.
		if maybe_replacement.max_fee_per_gas() <= self.max_fee_per_gas() {
			return true;
		}

		false
	}
}

#[cfg(test)]
impl<T: PoolTransaction> Clone for ValidPoolTransaction<T> {
	fn clone(&self) -> Self {
		Self {
			transaction: self.transaction.clone(),
			transaction_id: self.transaction_id,
			propagate: self.propagate,
			timestamp: self.timestamp,
			origin: self.origin,
		}
	}
}

impl<T: PoolTransaction> fmt::Debug for ValidPoolTransaction<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ValidPoolTransaction")
			.field("id", &self.transaction_id)
			.field("pragate", &self.propagate)
			.field("origin", &self.origin)
			.field("hash", self.transaction.hash())
			.field("tx", &self.transaction)
			.finish()
	}
}

/// Validation Errors that can occur during transaction validation.
#[derive(thiserror::Error, Debug)]
pub enum TransactionValidatorError {
	/// Failed to communicate with the validation service.
	#[error("validation service unreachable")]
	ValidationServiceUnreachable,
}
