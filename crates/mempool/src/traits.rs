use crate::{
	error::{InvalidPoolTransactionError, PoolError},
	util::{Address, Hash, TxHash},
	validate::ValidPoolTransaction,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	fmt::Debug,
	future::Future,
	sync::Arc,
};

/// The `PeerId` type.
pub type PeerId = u64;

/// Helper type alias to access [`PoolTransaction`] for a given [`TransactionPool`].
pub type PoolTx<P> = <P as TransactionPool>::Transaction;
/// Transaction pool result type.
pub type PoolResult<T> = Result<T, PoolError>;

/// General purpose abstraction of a transaction-pool.
///
/// This is intended to be used by API-consumers such as RPC that need inject new incoming,
/// unverified transactions. And by block production that needs to get transactions to execute in a
/// new block.
///
/// Note: This requires `Clone` for convenience, since it is assumed that this will be implemented
/// for a wrapped `Arc` type, see also [`Pool`](crate::Pool).
#[auto_impl::auto_impl(&, Arc)]
pub trait TransactionPool: Clone + Debug + Send + Sync {
	/// The transaction type of the pool
	type Transaction: PoolTransaction;

	/// Returns stats about the pool and all sub-pools.
	fn pool_size(&self) -> PoolSize;

	/// Returns the block the pool is currently tracking.
	///
	/// This tracks the block that the pool has last seen.
	fn block_info(&self) -> BlockInfo;

	/// Imports an _external_ transaction.
	///
	/// This is intended to be used by the network to insert incoming transactions received over the
	/// p2p network.
	///
	/// Consumer: P2P
	fn add_external_transaction(
		&self,
		transaction: Self::Transaction,
	) -> impl Future<Output = PoolResult<TxHash>> + Send {
		self.add_transaction(TransactionOrigin::External, transaction)
	}

	/// Imports all _external_ transactions
	///
	/// Consumer: Utility
	fn add_external_transactions(
		&self,
		transactions: Vec<Self::Transaction>,
	) -> impl Future<Output = Vec<PoolResult<TxHash>>> + Send {
		self.add_transactions(TransactionOrigin::External, transactions)
	}

	/// Adds an _unvalidated_ transaction into the pool.
	///
	/// Consumer: RPC
	fn add_transaction(
		&self,
		origin: TransactionOrigin,
		transaction: Self::Transaction,
	) -> impl Future<Output = PoolResult<TxHash>> + Send;

	/// Adds the given _unvalidated_ transaction into the pool.
	///
	/// Returns a list of results.
	///
	/// Consumer: RPC
	fn add_transactions(
		&self,
		origin: TransactionOrigin,
		transactions: Vec<Self::Transaction>,
	) -> impl Future<Output = Vec<PoolResult<TxHash>>> + Send;

	/// Note: This returns a `Vec` but should guarantee that all hashes are unique.
	///
	/// Consumer: P2P
	fn pooled_transaction_hashes(&self) -> Vec<TxHash>;

	/// Returns only the first `max` hashes of transactions in the pool.
	///
	/// Consumer: P2P
	fn pooled_transaction_hashes_max(&self, max: usize) -> Vec<TxHash>;

	/// Returns the _full_ transaction objects all transactions in the pool.
	///
	/// This is intended to be used by the network for the initial exchange of pooled transaction
	/// _hashes_
	///
	/// Note: This returns a `Vec` but should guarantee that all transactions are unique.
	///
	/// Caution: In case of blob transactions, this does not include the sidecar.
	///
	/// Consumer: P2P
	fn pooled_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns only the first `max` transactions in the pool.
	///
	/// Consumer: P2P
	fn pooled_transactions_max(
		&self,
		max: usize,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns an iterator that yields transactions that are ready for block production.
	///
	/// Consumer: Block production
	fn best_transactions(
		&self,
	) -> Box<dyn BestTransactions<Item = Arc<ValidPoolTransaction<Self::Transaction>>>>;

	/// Returns an iterator that yields transactions that are ready for block production with the
	/// given base fee and optional blob fee attributes.
	///
	/// Consumer: Block production
	fn best_transactions_with_attributes(
		&self,
		best_transactions_attributes: BestTransactionsAttributes,
	) -> Box<dyn BestTransactions<Item = Arc<ValidPoolTransaction<Self::Transaction>>>>;

	/// Returns all transactions that can be included in the next block.
	///
	/// This is primarily used for the `txpool_` RPC namespace:
	/// <https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-txpool> which distinguishes
	/// between `pending` and `queued` transactions, where `pending` are transactions ready for
	/// inclusion in the next block and `queued` are transactions that are ready for inclusion in
	/// future blocks.
	///
	/// Consumer: RPC
	fn pending_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns first `max` transactions that can be included in the next block.
	/// See <https://github.com/paradigmxyz/reth/issues/12767#issuecomment-2493223579>
	///
	/// Consumer: Block production
	fn pending_transactions_max(
		&self,
		max: usize,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all transactions that can be included in _future_ blocks.
	///
	/// This and [Self::pending_transactions] are mutually exclusive.
	///
	/// Consumer: RPC
	fn parked_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all transactions that are currently in the pool grouped by whether they are ready
	/// for inclusion in the next block or not.
	///
	/// This is primarily used for the `txpool_` namespace: <https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-txpool>
	///
	/// Consumer: RPC
	fn all_transactions(&self) -> AllPoolTransactions<Self::Transaction>;

	/// Removes all transactions corresponding to the given hashes.
	///
	/// Note: This removes the transactions as if they got discarded (_not_ mined).
	///
	/// Consumer: Utility
	fn remove_transactions(
		&self,
		hashes: Vec<TxHash>,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Removes all transactions corresponding to the given hashes.
	///
	/// Also removes all _dependent_ transactions.
	///
	/// Consumer: Utility
	fn remove_transactions_and_descendants(
		&self,
		hashes: Vec<TxHash>,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Removes all transactions from the given sender
	///
	/// Consumer: Utility
	fn remove_transactions_by_sender(
		&self,
		sender: Address,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns if the transaction for the given hash is already included in this pool.
	fn contains(&self, tx_hash: &TxHash) -> bool {
		self.get(tx_hash).is_some()
	}

	/// Returns the transaction for the given hash.
	fn get(&self, tx_hash: &TxHash) -> Option<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all transactions objects for the given hashes.
	///
	/// Caution: This in case of blob transactions, this does not include the sidecar.
	fn get_all(&self, txs: Vec<TxHash>) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Notify the pool about transactions that are propagated to peers.
	///
	/// Consumer: P2P
	fn on_propagated(&self, txs: PropagatedTransactions);

	/// Returns all transactions sent by a given user
	fn get_transactions_by_sender(
		&self,
		sender: Address,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all pending transactions filtered by predicate
	fn get_pending_transactions_with_predicate(
		&self,
		predicate: impl FnMut(&ValidPoolTransaction<Self::Transaction>) -> bool,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all pending transactions sent by a given user
	fn get_pending_transactions_by_sender(
		&self,
		sender: Address,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all queued transactions sent by a given user
	fn get_parked_transactions_by_sender(
		&self,
		sender: Address,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns the highest transaction sent by a given user
	fn get_highest_transaction_by_sender(
		&self,
		sender: Address,
	) -> Option<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns the transaction with the highest nonce that is executable given the on chain nonce.
	/// In other words the highest non nonce gapped transaction.
	///
	/// Note: The next pending pooled transaction must have the on chain nonce.
	///
	/// For example, for a given on chain nonce of `5`, the next transaction must have that nonce.
	/// If the pool contains txs `[5,6,7]` this returns tx `7`.
	/// If the pool contains txs `[6,7]` this returns `None` because the next valid nonce (5) is
	/// missing, which means txs `[6,7]` are nonce gapped.
	fn get_highest_consecutive_transaction_by_sender(
		&self,
		sender: Address,
		on_chain_nonce: u64,
	) -> Option<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns a transaction sent by a given user and a nonce
	fn get_transaction_by_sender_and_nonce(
		&self,
		sender: Address,
		nonce: u64,
	) -> Option<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all transactions that where submitted with the given [TransactionOrigin]
	fn get_transactions_by_origin(
		&self,
		origin: TransactionOrigin,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all pending transactions filtered by [`TransactionOrigin`]
	fn get_pending_transactions_by_origin(
		&self,
		origin: TransactionOrigin,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>>;

	/// Returns all transactions that where submitted as [TransactionOrigin::Local]
	fn get_local_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_transactions_by_origin(TransactionOrigin::Local)
	}

	/// Returns all transactions that where submitted as [TransactionOrigin::Private]
	fn get_private_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_transactions_by_origin(TransactionOrigin::Private)
	}

	/// Returns all transactions that where submitted as [TransactionOrigin::External]
	fn get_external_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_transactions_by_origin(TransactionOrigin::External)
	}

	/// Returns all pending transactions that where submitted as [TransactionOrigin::Local]
	fn get_local_pending_transactions(&self) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_pending_transactions_by_origin(TransactionOrigin::Local)
	}

	/// Returns all pending transactions that where submitted as [TransactionOrigin::Private]
	fn get_private_pending_transactions(
		&self,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_pending_transactions_by_origin(TransactionOrigin::Private)
	}

	/// Returns all pending transactions that where submitted as [TransactionOrigin::External]
	fn get_external_pending_transactions(
		&self,
	) -> Vec<Arc<ValidPoolTransaction<Self::Transaction>>> {
		self.get_pending_transactions_by_origin(TransactionOrigin::External)
	}

	/// Returns a set of all senders of transactions in the pool
	fn unique_senders(&self) -> HashSet<Address>;
}

/// Extension for [TransactionPool] trait that allows to set the current block info.
#[auto_impl::auto_impl(&, Arc)]
pub trait TransactionPoolExt: TransactionPool {}

/// A Helper type that bundles all transactions in the pool.
#[derive(Debug, Clone)]
pub struct AllPoolTransactions<T: PoolTransaction> {
	/// Transactions that are ready for inclusion in the next block.
	pub pending: Vec<Arc<ValidPoolTransaction<T>>>,
	/// Transactions that are ready for inclusion in _future_ blocks, but are currently parked,
	/// because they depend on other transactions that are not yet included in the pool (nonce gap)
	/// or otherwise blocked.
	pub parked: Vec<Arc<ValidPoolTransaction<T>>>,
}

// === impl AllPoolTransactions ===

impl<T: PoolTransaction> AllPoolTransactions<T> {
	/// Returns an iterator over all pending [`Recovered`] transactions.
	pub fn pending_recovered(&self) -> impl Iterator<Item = T> + '_ {
		self.pending.iter().map(|tx| tx.transaction.clone())
	}

	/// Returns an iterator over all parked [`Recovered`] transactions.
	pub fn parked_recovered(&self) -> impl Iterator<Item = T> + '_ {
		self.parked.iter().map(|tx| tx.transaction.clone())
	}

	/// Returns an iterator over all transactions, both pending and queued.
	pub fn all(&self) -> impl Iterator<Item = T> + '_ {
		self.pending.iter().chain(self.parked.iter()).map(|tx| tx.transaction.clone())
	}
}

impl<T: PoolTransaction> Default for AllPoolTransactions<T> {
	fn default() -> Self {
		Self { pending: Default::default(), parked: Default::default() }
	}
}

/// Represents transactions that were propagated over the network.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PropagatedTransactions(pub HashMap<TxHash, Vec<PropagateKind>>);

/// Represents how a transaction was propagated over the network.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PropagateKind {
	/// The full transaction object was sent to the peer.
	///
	/// This is equivalent to the `Transaction` message
	Full(PeerId),
	/// Only the Hash was propagated to the peer.
	Hash(PeerId),
}

// === impl PropagateKind ===

impl PropagateKind {
	/// Returns the peer the transaction was sent to
	pub const fn peer(&self) -> &PeerId {
		match self {
			Self::Full(peer) | Self::Hash(peer) => peer,
		}
	}

	/// Returns true if the transaction was sent as a full transaction
	pub const fn is_full(&self) -> bool {
		matches!(self, Self::Full(_))
	}

	/// Returns true if the transaction was sent as a hash
	pub const fn is_hash(&self) -> bool {
		matches!(self, Self::Hash(_))
	}
}

impl From<PropagateKind> for PeerId {
	fn from(value: PropagateKind) -> Self {
		match value {
			PropagateKind::Full(peer) | PropagateKind::Hash(peer) => peer,
		}
	}
}

/// Where the transaction originates from.
///
/// Depending on where the transaction was picked up, it affects how the transaction is handled
/// internally, e.g. limits for simultaneous transaction of one sender.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TransactionOrigin {
	/// Transaction is coming from a local source.
	#[default]
	Local,
	/// Transaction has been received externally.
	///
	/// This is usually considered an "untrusted" source, for example received from another in the
	/// network.
	External,
	/// Transaction is originated locally and is intended to remain private.
	///
	/// This type of transaction should not be propagated to the network. It's meant for
	/// private usage within the local node only.
	Private,
}

// === impl TransactionOrigin ===

impl TransactionOrigin {
	/// Whether the transaction originates from a local source.
	pub const fn is_local(&self) -> bool {
		matches!(self, Self::Local)
	}

	/// Whether the transaction originates from an external source.
	pub const fn is_external(&self) -> bool {
		matches!(self, Self::External)
	}
	/// Whether the transaction originates from a private source.
	pub const fn is_private(&self) -> bool {
		matches!(self, Self::Private)
	}
}

/// Represents changes after a new canonical block or range of canonical blocks was added to the
/// chain.
/// This is used to update the pool state accordingly.
#[derive(Clone, Debug)]
pub struct CanonicalStateUpdate {
	/// All mined transactions in the block range.
	pub mined_transactions: Vec<TxHash>,
}

/// Alias to restrict the [`BestTransactions`] items to the pool's transaction type.
pub type BestTransactionsFor<Pool> = Box<
	dyn BestTransactions<Item = Arc<ValidPoolTransaction<<Pool as TransactionPool>::Transaction>>>,
>;

/// An `Iterator` that only returns transactions that are ready to be executed.
///
/// This makes no assumptions about the order of the transactions, but expects that _all_
/// transactions are valid (no nonce gaps.) for the tracked state of the pool.
///
/// Note: this iterator will always return the best transaction that it currently knows.
/// There is no guarantee transactions will be returned sequentially in decreasing
/// priority order.
pub trait BestTransactions: Iterator + Send {
	/// Mark the transaction as invalid.
	///
	/// Implementers must ensure all subsequent transaction _don't_ depend on this transaction.
	/// In other words, this must remove the given transaction _and_ drain all transaction that
	/// depend on it.
	fn mark_invalid(&mut self, transaction: &Self::Item, kind: InvalidPoolTransactionError);

	/// An iterator may be able to receive additional pending transactions that weren't present it
	/// the pool when it was created.
	///
	/// This ensures that iterator will return the best transaction that it currently knows and not
	/// listen to pool updates.
	fn no_updates(&mut self);

	/// Convenience function for [`Self::no_updates`] that returns the iterator again.
	fn without_updates(mut self) -> Self
	where
		Self: Sized,
	{
		self.no_updates();
		self
	}
}

impl<T> BestTransactions for Box<T>
where
	T: BestTransactions + ?Sized,
{
	fn mark_invalid(&mut self, transaction: &Self::Item, kind: InvalidPoolTransactionError) {
		(**self).mark_invalid(transaction, kind)
	}

	fn no_updates(&mut self) {
		(**self).no_updates();
	}
}

/// A filter that allows to check if a transaction satisfies a set of conditions
pub trait TransactionFilter {
	/// The type of the transaction to check.
	type Transaction;

	/// Returns true if the transaction satisfies the conditions.
	fn is_valid(&self, transaction: &Self::Transaction) -> bool;
}

/// A Helper type that bundles the best transactions attributes together.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BestTransactionsAttributes {
	/// The fee attribute for best transactions.
	pub fee: u64,
}

// === impl BestTransactionsAttributes ===

impl BestTransactionsAttributes {
	/// Creates a new `BestTransactionsAttributes` with the given fee
	pub const fn new(fee: u64) -> Self {
		Self { fee }
	}

	/// Creates a new `BestTransactionsAttributes` with the given fee.
	pub const fn fee(fee: u64) -> Self {
		Self::new(fee)
	}
}

//
pub trait PoolTransaction: Debug + Send + Sync + Clone {
	/// Hash of the transaction.
	fn hash(&self) -> &TxHash;

	/// The Sender of the transaction.
	fn sender(&self) -> Address;

	/// Reference to the Sender of the transaction.
	fn sender_ref(&self) -> &Address;

	/// Returns the cost that this transaction is allowed to consume:
	fn cost(&self) -> u128;

	/// Returns the length of the rlp encoded transaction object
	fn encoded_length(&self) -> usize;

	fn max_fee_per_gas(&self) -> u128;

	fn gas(&self) -> u128;

	fn nonce(&self) -> u64;

	fn size(&self) -> usize;
}

/// Represents the current status of the pool.
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolSize {
	/// Number of transactions in the _pending_ sub-pool.
	pub pending: usize,
	/// Reported size of transactions in the _pending_ sub-pool.
	pub pending_size: usize,

	// Number of transactions in the _parked_ sub-pool.
	pub parked: usize,
	/// Reported size of transactions in the _parked_ sub-pool.
	pub parked_size: usize,
	/// Number of all transactions of all sub-pools
	///
	/// Note: this is the sum of ```pending + parked```
	pub total: usize,
}

// === impl PoolSize ===

impl PoolSize {
	/// Asserts that the invariants of the pool size are met.
	#[cfg(test)]
	pub(crate) fn assert_invariants(&self) {
		assert_eq!(self.total, self.pending + self.parked);
	}
}

/// Represents the current status of the pool.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct BlockInfo {
	/// Hash for the currently tracked block.
	pub last_seen_block_hash: Hash,
	/// Currently tracked block.
	pub last_seen_block_number: u64,
}

/// The limit to enforce for [`TransactionPool::get_pooled_transaction_elements`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GetPooledTransactionLimit {
	/// No limit, return all transactions.
	None,
	/// Enforce a size limit on the returned transactions, for example 2MB
	ResponseSizeSoftLimit(usize),
}

impl GetPooledTransactionLimit {
	/// Returns true if the given size exceeds the limit.
	#[inline]
	pub const fn exceeds(&self, size: usize) -> bool {
		match self {
			Self::None => false,
			Self::ResponseSizeSoftLimit(limit) => size > *limit,
		}
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use alloy_consensus::{
// 		EthereumTxEnvelope, SignableTransaction, TxEip1559, TxEip2930, TxEip4844, TxEip7702,
// 		TxEnvelope, TxLegacy,
// 	};
// 	use alloy_eips::eip4844::DATA_GAS_PER_BLOB;
// 	use alloy_primitives::Signature;

// 	#[test]
// 	fn test_pool_size_invariants() {
// 		let pool_size = PoolSize {
// 			pending: 10,
// 			pending_size: 1000,
// 			basefee: 8,
// 			basefee_size: 800,
// 			queued: 7,
// 			queued_size: 700,
// 			total: 10 + 5 + 8 + 7, // Correct total
// 		};

// 		// Call the assert_invariants method to check if the invariants are correct
// 		pool_size.assert_invariants();
// 	}

// 	#[test]
// 	#[should_panic]
// 	fn test_pool_size_invariants_fail() {
// 		let pool_size = PoolSize {
// 			pending: 10,
// 			pending_size: 1000,
// 			basefee: 8,
// 			basefee_size: 800,
// 			queued: 7,
// 			queued_size: 700,
// 			total: 10 + 5 + 8, // Incorrect total
// 		};

// 		// Call the assert_invariants method, which should panic
// 		pool_size.assert_invariants();
// 	}

// 	#[test]
// 	fn test_eth_pooled_transaction_new_legacy() {
// 		// Create a legacy transaction with specific parameters
// 		let tx = TxEnvelope::Legacy(
// 			TxLegacy {
// 				gas_price: 10,
// 				gas_limit: 1000,
// 				value: U256::from(100),
// 				..Default::default()
// 			}
// 			.into_signed(Signature::test_signature()),
// 		);
// 		let transaction = Recovered::new_unchecked(tx, Default::default());
// 		let pooled_tx = EthPooledTransaction::new(transaction.clone(), 200);

// 		// Check that the pooled transaction is created correctly
// 		assert_eq!(pooled_tx.transaction, transaction);
// 		assert_eq!(pooled_tx.encoded_length, 200);
// 		assert_eq!(pooled_tx.blob_sidecar, EthBlobTransactionSidecar::None);
// 		assert_eq!(pooled_tx.cost, U256::from(100) + U256::from(10 * 1000));
// 	}

// 	#[test]
// 	fn test_eth_pooled_transaction_new_eip2930() {
// 		// Create an EIP-2930 transaction with specific parameters
// 		let tx = TxEnvelope::Eip2930(
// 			TxEip2930 {
// 				gas_price: 10,
// 				gas_limit: 1000,
// 				value: U256::from(100),
// 				..Default::default()
// 			}
// 			.into_signed(Signature::test_signature()),
// 		);
// 		let transaction = Recovered::new_unchecked(tx, Default::default());
// 		let pooled_tx = EthPooledTransaction::new(transaction.clone(), 200);
// 		let expected_cost = U256::from(100) + (U256::from(10 * 1000));

// 		assert_eq!(pooled_tx.transaction, transaction);
// 		assert_eq!(pooled_tx.encoded_length, 200);
// 		assert_eq!(pooled_tx.blob_sidecar, EthBlobTransactionSidecar::None);
// 		assert_eq!(pooled_tx.cost, expected_cost);
// 	}

// 	#[test]
// 	fn test_eth_pooled_transaction_new_eip1559() {
// 		// Create an EIP-1559 transaction with specific parameters
// 		let tx = TxEnvelope::Eip1559(
// 			TxEip1559 {
// 				max_fee_per_gas: 10,
// 				gas_limit: 1000,
// 				value: U256::from(100),
// 				..Default::default()
// 			}
// 			.into_signed(Signature::test_signature()),
// 		);
// 		let transaction = Recovered::new_unchecked(tx, Default::default());
// 		let pooled_tx = EthPooledTransaction::new(transaction.clone(), 200);

// 		// Check that the pooled transaction is created correctly
// 		assert_eq!(pooled_tx.transaction, transaction);
// 		assert_eq!(pooled_tx.encoded_length, 200);
// 		assert_eq!(pooled_tx.blob_sidecar, EthBlobTransactionSidecar::None);
// 		assert_eq!(pooled_tx.cost, U256::from(100) + U256::from(10 * 1000));
// 	}

// 	#[test]
// 	fn test_eth_pooled_transaction_new_eip4844() {
// 		// Create an EIP-4844 transaction with specific parameters
// 		let tx = EthereumTxEnvelope::Eip4844(
// 			TxEip4844 {
// 				max_fee_per_gas: 10,
// 				gas_limit: 1000,
// 				value: U256::from(100),
// 				max_fee_per_blob_gas: 5,
// 				blob_versioned_hashes: vec![B256::default()],
// 				..Default::default()
// 			}
// 			.into_signed(Signature::test_signature()),
// 		);
// 		let transaction = Recovered::new_unchecked(tx, Default::default());
// 		let pooled_tx = EthPooledTransaction::new(transaction.clone(), 300);

// 		// Check that the pooled transaction is created correctly
// 		assert_eq!(pooled_tx.transaction, transaction);
// 		assert_eq!(pooled_tx.encoded_length, 300);
// 		assert_eq!(pooled_tx.blob_sidecar, EthBlobTransactionSidecar::Missing);
// 		let expected_cost =
// 			U256::from(100) + U256::from(10 * 1000) + U256::from(5 * DATA_GAS_PER_BLOB);
// 		assert_eq!(pooled_tx.cost, expected_cost);
// 	}

// 	#[test]
// 	fn test_eth_pooled_transaction_new_eip7702() {
// 		// Init an EIP-7702 transaction with specific parameters
// 		let tx = EthereumTxEnvelope::<TxEip4844>::Eip7702(
// 			TxEip7702 {
// 				max_fee_per_gas: 10,
// 				gas_limit: 1000,
// 				value: U256::from(100),
// 				..Default::default()
// 			}
// 			.into_signed(Signature::test_signature()),
// 		);
// 		let transaction = Recovered::new_unchecked(tx, Default::default());
// 		let pooled_tx = EthPooledTransaction::new(transaction.clone(), 200);

// 		// Check that the pooled transaction is created correctly
// 		assert_eq!(pooled_tx.transaction, transaction);
// 		assert_eq!(pooled_tx.encoded_length, 200);
// 		assert_eq!(pooled_tx.blob_sidecar, EthBlobTransactionSidecar::None);
// 		assert_eq!(pooled_tx.cost, U256::from(100) + U256::from(10 * 1000));
// 	}

// 	#[test]
// 	fn test_pooled_transaction_limit() {
// 		// No limit should never exceed
// 		let limit_none = GetPooledTransactionLimit::None;
// 		// Any size should return false
// 		assert!(!limit_none.exceeds(1000));

// 		// Size limit of 2MB (2 * 1024 * 1024 bytes)
// 		let size_limit_2mb = GetPooledTransactionLimit::ResponseSizeSoftLimit(2 * 1024 * 1024);

// 		// Test with size below the limit
// 		// 1MB is below 2MB, should return false
// 		assert!(!size_limit_2mb.exceeds(1024 * 1024));

// 		// Test with size exactly at the limit
// 		// 2MB equals the limit, should return false
// 		assert!(!size_limit_2mb.exceeds(2 * 1024 * 1024));

// 		// Test with size exceeding the limit
// 		// 3MB is above the 2MB limit, should return true
// 		assert!(size_limit_2mb.exceeds(3 * 1024 * 1024));
// 	}
// }
