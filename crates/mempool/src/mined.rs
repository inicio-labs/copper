use std::sync::Arc;

use crate::{
	state::SubPool,
	traits::PoolTransaction,
	util::{Hash, TxHash},
	validate::ValidPoolTransaction,
};

#[derive(Debug, Clone)]
pub struct AddedPendingTransaction<T: PoolTransaction> {
	/// Inserted transaction.
	pub transaction: Arc<ValidPoolTransaction<T>>,
	/// Replaced transaction.
	pub replaced: Option<Arc<ValidPoolTransaction<T>>>,
	/// transactions promoted to the pending queue
	pub promoted: Vec<Arc<ValidPoolTransaction<T>>>,
	/// transactions that failed and became discarded
	pub discarded: Vec<Arc<ValidPoolTransaction<T>>>,
}

impl<T: PoolTransaction> AddedPendingTransaction<T> {
	/// Returns if the transaction should be propagated.
	pub(crate) fn is_propagate_allowed(&self) -> bool {
		self.transaction.propagate
	}
}
/// Represents a transaction that was added into the pool and its state
#[derive(Debug, Clone)]
pub enum AddedTransaction<T: PoolTransaction> {
	/// Transaction was successfully added and moved to the pending pool.
	Pending(AddedPendingTransaction<T>),
	/// Transaction was successfully added but not yet ready for processing and moved to a
	/// parked pool instead.
	Parked {
		/// Inserted transaction.
		transaction: Arc<ValidPoolTransaction<T>>,
		/// Replaced transaction.
		replaced: Option<Arc<ValidPoolTransaction<T>>>,
		/// The subpool it was moved to.
		subpool: SubPool,
	},
}

impl<T: PoolTransaction> AddedTransaction<T> {
	/// Returns whether the transaction has been added to the pending pool.
	pub(crate) const fn as_pending(&self) -> Option<&AddedPendingTransaction<T>> {
		match self {
			Self::Pending(tx) => Some(tx),
			_ => None,
		}
	}

	/// Returns the replaced transaction if there was one
	pub(crate) const fn replaced(&self) -> Option<&Arc<ValidPoolTransaction<T>>> {
		match self {
			Self::Pending(tx) => tx.replaced.as_ref(),
			Self::Parked { replaced, .. } => replaced.as_ref(),
		}
	}

	/// Returns the discarded transactions if there were any
	pub(crate) fn discarded_transactions(&self) -> Option<&[Arc<ValidPoolTransaction<T>>]> {
		match self {
			Self::Pending(tx) => Some(&tx.discarded),
			Self::Parked { .. } => None,
		}
	}

	/// Returns the hash of the transaction
	pub(crate) fn hash(&self) -> &TxHash {
		match self {
			Self::Pending(tx) => tx.transaction.hash(),
			Self::Parked { transaction, .. } => transaction.hash(),
		}
	}
}

pub(crate) struct OnNewCanonicalStateOutcome<T: PoolTransaction> {
	/// Hash of the block.
	pub(crate) block_hash: Hash,
	/// All mined transactions.
	pub(crate) mined: Vec<TxHash>,
	/// Transactions promoted to the pending pool.
	pub(crate) promoted: Vec<Arc<ValidPoolTransaction<T>>>,
	/// transaction that were discarded during the update
	pub(crate) discarded: Vec<Arc<ValidPoolTransaction<T>>>,
}
