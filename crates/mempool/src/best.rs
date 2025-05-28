use std::{
	collections::{BTreeMap, BTreeSet, HashSet},
	sync::Arc,
};

use tokio::sync::broadcast::Receiver;

use crate::{
	error::InvalidPoolTransactionError,
	identifier::{SenderId, TransactionId},
	ordering::TransactionOrdering,
	pending::PendingTransaction,
	validate::ValidPoolTransaction,
};

#[derive(Debug)]
pub struct BestTransactions<T: TransactionOrdering> {
	/// Contains a copy of _all_ transactions of the pending pool at the point in time this
	/// iterator was created.
	pub(crate) all: BTreeMap<TransactionId, PendingTransaction<T>>,
	/// Transactions that can be executed right away: these have the expected nonce.
	///
	/// Once an `independent` transaction with the nonce `N` is returned, it unlocks `N+1`, which
	/// then can be moved from the `all` set to the `independent` set.
	pub(crate) independent: BTreeSet<PendingTransaction<T>>,
	/// There might be the case where a yielded transactions is invalid, this will track it.
	pub(crate) invalid: HashSet<SenderId>,

	pub(crate) new_transaction_receiver: Option<Receiver<PendingTransaction<T>>>,
}

impl<T: TransactionOrdering> BestTransactions<T> {
	/// Mark the transaction and it's descendants as invalid.
	pub(crate) fn mark_invalid(
		&mut self,
		tx: &Arc<ValidPoolTransaction<T::Transaction>>,
		_kind: InvalidPoolTransactionError,
	) {
		self.invalid.insert(tx.sender_id());
	}

	/// Returns the ancestor the given transaction, the transaction with `nonce - 1`.
	///
	/// Note: for a transaction with nonce higher than the current on chain nonce this will always
	/// return an ancestor since all transaction in this pool are gapless.
	pub(crate) fn ancestor(&self, id: &TransactionId) -> Option<&PendingTransaction<T>> {
		self.all.get(&id.unchecked_ancestor()?)
	}

	/// Non-blocking read on the new pending transactions subscription channel
	fn try_recv(&mut self) -> Option<PendingTransaction<T>> {
		loop {
			match self.new_transaction_receiver.as_mut()?.try_recv() {
				Ok(tx) => return Some(tx),
				// note TryRecvError::Lagged can be returned here, which is an error that attempts
				// to correct itself on consecutive try_recv() attempts

				//TODO: handle this error
				// // the cost of ignoring this error is allowing old transactions to get
				// // overwritten after the chan buffer size is met
				// Err(TryRecvError::Lagged(_)) => {
				// 	// Handle the case where the receiver lagged too far behind.
				// 	// `num_skipped` indicates the number of messages that were skipped.
				// },

				// this case is still better than the existing iterator behavior where no new
				// pending txs are surfaced to consumers
				Err(_) => return None,
			}
		}
	}

	/// Removes the currently best independent transaction from the independent set and the total
	/// set.
	fn pop_best(&mut self) -> Option<PendingTransaction<T>> {
		self.independent.pop_last().inspect(|best| {
			self.all.remove(best.transaction.id());
		})
	}

	/// Checks for new transactions that have come into the `PendingPool` after this iterator was
	/// created and inserts them
	fn add_new_transactions(&mut self) {
		while let Some(pending_tx) = self.try_recv() {
			//  same logic as PendingPool::add_transaction/PendingPool::best_with_unlocked
			let tx_id = *pending_tx.transaction.id();
			if self.ancestor(&tx_id).is_none() {
				self.independent.insert(pending_tx.clone());
			}
			self.all.insert(tx_id, pending_tx);
		}
	}
}
