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

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use crate::{
// 		BestTransactions, Priority,
// 		pool::pending::PendingPool,
// 		test_utils::{MockOrdering, MockTransaction, MockTransactionFactory},
// 	};
// 	use alloy_primitives::U256;

// 	#[test]
// 	fn test_best_iter() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 10;
// 		// insert 10 gapless tx
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best();
// 		assert_eq!(best.all.len(), num_tx as usize);
// 		assert_eq!(best.independent.len(), 1);

// 		// check tx are returned in order
// 		for nonce in 0..num_tx {
// 			assert_eq!(best.independent.len(), 1);
// 			let tx = best.next().unwrap();
// 			assert_eq!(tx.nonce(), nonce);
// 		}
// 	}

// 	#[test]
// 	fn test_best_iter_invalid() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 10;
// 		// insert 10 gapless tx
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best();

// 		// mark the first tx as invalid
// 		let invalid = best.independent.iter().next().unwrap();
// 		best.mark_invalid(
// 			&invalid.transaction.clone(),
// 			InvalidPoolTransactionError::Consensus(InvalidTransactionError::TxTypeNotSupported),
// 		);

// 		// iterator is empty
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_transactions_iter_invalid() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 10;
// 		// insert 10 gapless tx
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best: Box<
// 			dyn crate::traits::BestTransactions<Item = Arc<ValidPoolTransaction<MockTransaction>>>,
// 		> = Box::new(pool.best());

// 		let tx = Iterator::next(&mut best).unwrap();
// 		crate::traits::BestTransactions::mark_invalid(
// 			&mut *best,
// 			&tx,
// 			InvalidPoolTransactionError::Consensus(InvalidTransactionError::TxTypeNotSupported),
// 		);
// 		assert!(Iterator::next(&mut best).is_none());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_base_fee_satisfied() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 5;
// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 15;

// 		// Insert transactions with a max_fee_per_gas greater than or equal to the base fee
// 		// Without blob fee
// 		for nonce in 0..num_tx {
// 			let tx = MockTransaction::eip1559()
// 				.rng_hash()
// 				.with_nonce(nonce)
// 				.with_max_fee(base_fee as u128 + 5);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		for nonce in 0..num_tx {
// 			let tx = best.next().expect("Transaction should be returned");
// 			assert_eq!(tx.nonce(), nonce);
// 			assert!(tx.transaction.max_fee_per_gas() >= base_fee as u128);
// 		}
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_base_fee_violated() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 5;
// 		let base_fee: u64 = 20;
// 		let base_fee_per_blob_gas: u64 = 15;

// 		// Insert transactions with a max_fee_per_gas less than the base fee
// 		for nonce in 0..num_tx {
// 			let tx = MockTransaction::eip1559()
// 				.rng_hash()
// 				.with_nonce(nonce)
// 				.with_max_fee(base_fee as u128 - 5);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		// No transaction should be returned since all violate the base fee
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_blob_fee_satisfied() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 5;
// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 20;

// 		// Insert transactions with a max_fee_per_blob_gas greater than or equal to the base fee per
// 		// blob gas
// 		for nonce in 0..num_tx {
// 			let tx = MockTransaction::eip4844()
// 				.rng_hash()
// 				.with_nonce(nonce)
// 				.with_max_fee(base_fee as u128 + 5)
// 				.with_blob_fee(base_fee_per_blob_gas as u128 + 5);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		// All transactions should be returned in order since they satisfy both base fee and blob
// 		// fee
// 		for nonce in 0..num_tx {
// 			let tx = best.next().expect("Transaction should be returned");
// 			assert_eq!(tx.nonce(), nonce);
// 			assert!(tx.transaction.max_fee_per_gas() >= base_fee as u128);
// 			assert!(
// 				tx.transaction.max_fee_per_blob_gas().unwrap() >= base_fee_per_blob_gas as u128
// 			);
// 		}

// 		// No more transactions should be returned
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_blob_fee_violated() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let num_tx = 5;
// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 20;

// 		// Insert transactions with a max_fee_per_blob_gas less than the base fee per blob gas
// 		for nonce in 0..num_tx {
// 			let tx = MockTransaction::eip4844()
// 				.rng_hash()
// 				.with_nonce(nonce)
// 				.with_max_fee(base_fee as u128 + 5)
// 				.with_blob_fee(base_fee_per_blob_gas as u128 - 5);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		// No transaction should be returned since all violate the blob fee
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_mixed_fees() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 20;

// 		// Insert transactions with varying max_fee_per_gas and max_fee_per_blob_gas
// 		let tx1 =
// 			MockTransaction::eip1559().rng_hash().with_nonce(0).with_max_fee(base_fee as u128 + 5);
// 		let tx2 = MockTransaction::eip4844()
// 			.rng_hash()
// 			.with_nonce(1)
// 			.with_max_fee(base_fee as u128 + 5)
// 			.with_blob_fee(base_fee_per_blob_gas as u128 + 5);
// 		let tx3 = MockTransaction::eip4844()
// 			.rng_hash()
// 			.with_nonce(2)
// 			.with_max_fee(base_fee as u128 + 5)
// 			.with_blob_fee(base_fee_per_blob_gas as u128 - 5);
// 		let tx4 =
// 			MockTransaction::eip1559().rng_hash().with_nonce(3).with_max_fee(base_fee as u128 - 5);

// 		pool.add_transaction(Arc::new(f.validated(tx1.clone())), 0);
// 		pool.add_transaction(Arc::new(f.validated(tx2.clone())), 0);
// 		pool.add_transaction(Arc::new(f.validated(tx3)), 0);
// 		pool.add_transaction(Arc::new(f.validated(tx4)), 0);

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		let expected_order = vec![tx1, tx2];
// 		for expected_tx in expected_order {
// 			let tx = best.next().expect("Transaction should be returned");
// 			assert_eq!(tx.transaction, expected_tx);
// 		}

// 		// No more transactions should be returned
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_add_transaction_with_next_nonce() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add 5 transactions with increasing nonces to the pool
// 		let num_tx = 5;
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		// Create a BestTransactions iterator from the pool
// 		let mut best = pool.best();

// 		// Use a broadcast channel for transaction updates
// 		let (tx_sender, tx_receiver) =
// 			tokio::sync::broadcast::channel::<PendingTransaction<MockOrdering>>(1000);
// 		best.new_transaction_receiver = Some(tx_receiver);

// 		// Create a new transaction with nonce 5 and validate it
// 		let new_tx = MockTransaction::eip1559().rng_hash().with_nonce(5);
// 		let valid_new_tx = f.validated(new_tx);

// 		// Send the new transaction through the broadcast channel
// 		let pending_tx = PendingTransaction {
// 			submission_id: 10,
// 			transaction: Arc::new(valid_new_tx.clone()),
// 			priority: Priority::Value(U256::from(1000)),
// 		};
// 		tx_sender.send(pending_tx.clone()).unwrap();

// 		// Add new transactions to the iterator
// 		best.add_new_transactions();

// 		// Verify that the new transaction has been added to the 'all' map
// 		assert_eq!(best.all.len(), 6);
// 		assert!(best.all.contains_key(valid_new_tx.id()));

// 		// Verify that the new transaction has been added to the 'independent' set
// 		assert_eq!(best.independent.len(), 2);
// 		assert!(best.independent.contains(&pending_tx));
// 	}

// 	#[test]
// 	fn test_best_add_transaction_with_ancestor() {
// 		// Initialize a new PendingPool with default MockOrdering and MockTransactionFactory
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add 5 transactions with increasing nonces to the pool
// 		let num_tx = 5;
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		// Create a BestTransactions iterator from the pool
// 		let mut best = pool.best();

// 		// Use a broadcast channel for transaction updates
// 		let (tx_sender, tx_receiver) =
// 			tokio::sync::broadcast::channel::<PendingTransaction<MockOrdering>>(1000);
// 		best.new_transaction_receiver = Some(tx_receiver);

// 		// Create a new transaction with nonce 5 and validate it
// 		let base_tx1 = MockTransaction::eip1559().rng_hash().with_nonce(5);
// 		let valid_new_tx1 = f.validated(base_tx1.clone());

// 		// Send the new transaction through the broadcast channel
// 		let pending_tx1 = PendingTransaction {
// 			submission_id: 10,
// 			transaction: Arc::new(valid_new_tx1.clone()),
// 			priority: Priority::Value(U256::from(1000)),
// 		};
// 		tx_sender.send(pending_tx1.clone()).unwrap();

// 		// Add new transactions to the iterator
// 		best.add_new_transactions();

// 		// Verify that the new transaction has been added to the 'all' map
// 		assert_eq!(best.all.len(), 6);
// 		assert!(best.all.contains_key(valid_new_tx1.id()));

// 		// Verify that the new transaction has been added to the 'independent' set
// 		assert_eq!(best.independent.len(), 2);
// 		assert!(best.independent.contains(&pending_tx1));

// 		// Attempt to add a new transaction with a different nonce (not a duplicate)
// 		let base_tx2 = base_tx1.with_nonce(6);
// 		let valid_new_tx2 = f.validated(base_tx2);

// 		// Send the new transaction through the broadcast channel
// 		let pending_tx2 = PendingTransaction {
// 			submission_id: 11, // Different submission ID
// 			transaction: Arc::new(valid_new_tx2.clone()),
// 			priority: Priority::Value(U256::from(1000)),
// 		};
// 		tx_sender.send(pending_tx2.clone()).unwrap();

// 		// Add new transactions to the iterator
// 		best.add_new_transactions();

// 		// Verify that the new transaction has been added to 'all'
// 		assert_eq!(best.all.len(), 7);
// 		assert!(best.all.contains_key(valid_new_tx2.id()));

// 		// Verify that the new transaction has not been added to the 'independent' set
// 		assert_eq!(best.independent.len(), 2);
// 		assert!(!best.independent.contains(&pending_tx2));
// 	}

// 	#[test]
// 	fn test_best_transactions_filter_trait_object() {
// 		// Initialize a new PendingPool with default MockOrdering and MockTransactionFactory
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add 5 transactions with increasing nonces to the pool
// 		let num_tx = 5;
// 		let tx = MockTransaction::eip1559();
// 		for nonce in 0..num_tx {
// 			let tx = tx.clone().rng_hash().with_nonce(nonce);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		// Create a trait object of BestTransactions iterator from the pool
// 		let best: Box<dyn crate::traits::BestTransactions<Item = _>> = Box::new(pool.best());

// 		// Create a filter that only returns transactions with even nonces
// 		let filter =
// 			BestTransactionFilter::new(best, |tx: &Arc<ValidPoolTransaction<MockTransaction>>| {
// 				tx.nonce() % 2 == 0
// 			});

// 		// Verify that the filter only returns transactions with even nonces
// 		for tx in filter {
// 			assert_eq!(tx.nonce() % 2, 0);
// 		}
// 	}

// 	#[test]
// 	fn test_best_transactions_prioritized_senders() {
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add 5 plain transactions from different senders with increasing gas price
// 		for gas_price in 0..5 {
// 			let tx = MockTransaction::eip1559().with_gas_price((gas_price + 1) * 10);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		// Add another transaction with 5 gas price that's going to be prioritized by sender
// 		let prioritized_tx = MockTransaction::eip1559().with_gas_price(5).with_gas_limit(200);
// 		let valid_prioritized_tx = f.validated(prioritized_tx.clone());
// 		pool.add_transaction(Arc::new(valid_prioritized_tx), 0);

// 		// Add another transaction with 3 gas price that should not be prioritized by sender because
// 		// of gas limit.
// 		let prioritized_tx2 = MockTransaction::eip1559().with_gas_price(3);
// 		let valid_prioritized_tx2 = f.validated(prioritized_tx2.clone());
// 		pool.add_transaction(Arc::new(valid_prioritized_tx2), 0);

// 		let prioritized_senders =
// 			HashSet::from([prioritized_tx.sender(), prioritized_tx2.sender()]);
// 		let best =
// 			BestTransactionsWithPrioritizedSenders::new(prioritized_senders, 200, pool.best());

// 		// Verify that the prioritized transaction is returned first
// 		// and the rest are returned in the reverse order of gas price
// 		let mut iter = best.into_iter();
// 		let top_of_block_tx = iter.next().unwrap();
// 		assert_eq!(top_of_block_tx.max_fee_per_gas(), 5);
// 		assert_eq!(top_of_block_tx.sender(), prioritized_tx.sender());
// 		for gas_price in (0..5).rev() {
// 			assert_eq!(iter.next().unwrap().max_fee_per_gas(), (gas_price + 1) * 10);
// 		}

// 		// Due to the gas limit, the transaction from second prioritized sender was not
// 		// prioritized.
// 		let top_of_block_tx2 = iter.next().unwrap();
// 		assert_eq!(top_of_block_tx2.max_fee_per_gas(), 3);
// 		assert_eq!(top_of_block_tx2.sender(), prioritized_tx2.sender());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_no_blob_fee_required() {
// 		// Tests transactions without blob fees where base fees are checked.
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 0; // No blob fee requirement

// 		// Insert transactions with max_fee_per_gas above the base fee
// 		for nonce in 0..5 {
// 			let tx = MockTransaction::eip1559()
// 				.rng_hash()
// 				.with_nonce(nonce)
// 				.with_max_fee(base_fee as u128 + 5);
// 			let valid_tx = f.validated(tx);
// 			pool.add_transaction(Arc::new(valid_tx), 0);
// 		}

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		// All transactions should be returned as no blob fee requirement is imposed
// 		for nonce in 0..5 {
// 			let tx = best.next().expect("Transaction should be returned");
// 			assert_eq!(tx.nonce(), nonce);
// 		}

// 		// Ensure no more transactions are left
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_with_fees_iter_mix_of_blob_and_non_blob_transactions() {
// 		// Tests mixed scenarios with both blob and non-blob transactions.
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		let base_fee: u64 = 10;
// 		let base_fee_per_blob_gas: u64 = 15;

// 		// Add a non-blob transaction that satisfies the base fee
// 		let tx_non_blob =
// 			MockTransaction::eip1559().rng_hash().with_nonce(0).with_max_fee(base_fee as u128 + 5);
// 		pool.add_transaction(Arc::new(f.validated(tx_non_blob.clone())), 0);

// 		// Add a blob transaction that satisfies both base fee and blob fee
// 		let tx_blob = MockTransaction::eip4844()
// 			.rng_hash()
// 			.with_nonce(1)
// 			.with_max_fee(base_fee as u128 + 5)
// 			.with_blob_fee(base_fee_per_blob_gas as u128 + 5);
// 		pool.add_transaction(Arc::new(f.validated(tx_blob.clone())), 0);

// 		let mut best = pool.best_with_basefee_and_blobfee(base_fee, base_fee_per_blob_gas);

// 		// Verify both transactions are returned
// 		let tx = best.next().expect("Transaction should be returned");
// 		assert_eq!(tx.transaction, tx_non_blob);

// 		let tx = best.next().expect("Transaction should be returned");
// 		assert_eq!(tx.transaction, tx_blob);

// 		// Ensure no more transactions are left
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_transactions_with_skipping_blobs() {
// 		// Tests the skip_blobs functionality to ensure blob transactions are skipped.
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add a blob transaction
// 		let tx_blob = MockTransaction::eip4844().rng_hash().with_nonce(0).with_blob_fee(100);
// 		let valid_blob_tx = f.validated(tx_blob);
// 		pool.add_transaction(Arc::new(valid_blob_tx), 0);

// 		// Add a non-blob transaction
// 		let tx_non_blob = MockTransaction::eip1559().rng_hash().with_nonce(1).with_max_fee(200);
// 		let valid_non_blob_tx = f.validated(tx_non_blob.clone());
// 		pool.add_transaction(Arc::new(valid_non_blob_tx), 0);

// 		let mut best = pool.best();
// 		best.skip_blobs();

// 		// Only the non-blob transaction should be returned
// 		let tx = best.next().expect("Transaction should be returned");
// 		assert_eq!(tx.transaction, tx_non_blob);

// 		// Ensure no more transactions are left
// 		assert!(best.next().is_none());
// 	}

// 	#[test]
// 	fn test_best_transactions_no_updates() {
// 		// Tests the no_updates functionality to ensure it properly clears the
// 		// new_transaction_receiver.
// 		let mut pool = PendingPool::new(MockOrdering::default());
// 		let mut f = MockTransactionFactory::default();

// 		// Add a transaction
// 		let tx = MockTransaction::eip1559().rng_hash().with_nonce(0).with_max_fee(100);
// 		let valid_tx = f.validated(tx);
// 		pool.add_transaction(Arc::new(valid_tx), 0);

// 		let mut best = pool.best();

// 		// Use a broadcast channel for transaction updates
// 		let (_tx_sender, tx_receiver) =
// 			tokio::sync::broadcast::channel::<PendingTransaction<MockOrdering>>(1000);
// 		best.new_transaction_receiver = Some(tx_receiver);

// 		// Ensure receiver is set
// 		assert!(best.new_transaction_receiver.is_some());

// 		// Call no_updates to clear the receiver
// 		best.no_updates();

// 		// Ensure receiver is cleared
// 		assert!(best.new_transaction_receiver.is_none());
// 	}

// 	// TODO: Same nonce test
// }
