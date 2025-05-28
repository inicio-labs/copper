use std::{collections::HashSet, num::NonZeroUsize, ops::Mul, path, time::Duration};

use crate::{
	traits::{PoolSize, TransactionOrigin},
	util::Address,
};

// Config constants
pub const TXPOOL_SUBPOOL_MAX_TXS_DEFAULT: usize = 1000;
pub const TXPOOL_SUBPOOL_MAX_SIZE_MB_DEFAULT: usize = 20;
pub const TXPOOL_MAX_ACCOUNT_SLOTS_PER_SENDER: usize = 100;
pub const MIN_PROTOCOL_BASE_FEE: u128 = 100;
pub const BLOCK_GAS_LIMIT: u128 = 30_000_000;
pub const MAX_QUEUED_TRANSACTION_LIFETIME: Duration = Duration::from_secs(3 * 24 * 60 * 60); // 3 days

pub struct MempoolConfig {
	// max size of tx
	max_tx_bytes: NonZeroUsize,

	// max number of txs in pending mempool
	pendingsize: NonZeroUsize,

	// max size of commulative txs bytes in pending mempool
	pending_max_txs_bytes: NonZeroUsize,

	// max number of txs in parked mempool
	parked_size: NonZeroUsize,

	// max size of commulative txs bytes in parked mempool
	parked_max_txs_bytes: NonZeroUsize,

	// wal path for temp mempool data storage
	wal_path: path::PathBuf,

	// broadcast txs to other nodes, if false,it will not broadcast txs to other nodes
	broadcast: bool,
}

impl Default for MempoolConfig {
	fn default() -> Self {
		Self {
			max_tx_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
			pendingsize: NonZeroUsize::new(TXPOOL_SUBPOOL_MAX_TXS_DEFAULT).unwrap(),
			pending_max_txs_bytes: NonZeroUsize::new(TXPOOL_SUBPOOL_MAX_TXS_DEFAULT * 1024 * 1024)
				.unwrap(),
			parked_size: NonZeroUsize::new(TXPOOL_SUBPOOL_MAX_TXS_DEFAULT).unwrap(),
			parked_max_txs_bytes: NonZeroUsize::new(TXPOOL_SUBPOOL_MAX_TXS_DEFAULT * 1024 * 1024)
				.unwrap(),
			wal_path: path::PathBuf::from(""),
			broadcast: true,
		}
	}
}

impl MempoolConfig {
	pub fn new(
		max_tx_bytes: NonZeroUsize,
		pendingsize: NonZeroUsize,
		pending_max_txs_bytes: NonZeroUsize,
		parked_size: NonZeroUsize,
		parked_max_txs_bytes: NonZeroUsize,
		wal_path: path::PathBuf,
		broadcast: bool,
	) -> Self {
		Self {
			max_tx_bytes,
			pendingsize,
			pending_max_txs_bytes,
			parked_size,
			parked_max_txs_bytes,
			wal_path,
			broadcast,
		}
	}

	pub fn max_tx_bytes(&self) -> NonZeroUsize {
		self.max_tx_bytes
	}

	pub fn pending_max_txs_bytes(&self) -> NonZeroUsize {
		self.pending_max_txs_bytes
	}

	pub fn pending_size(&self) -> NonZeroUsize {
		self.pendingsize
	}

	pub fn parked_size(&self) -> NonZeroUsize {
		self.parked_size
	}

	pub fn parked_max_txs_bytes(&self) -> NonZeroUsize {
		self.parked_max_txs_bytes
	}

	pub fn wal_path(&self) -> &path::Path {
		&self.wal_path
	}

	pub fn is_exceeded(&self, pool_size: PoolSize) -> bool {
		self.pendingsize.get() < pool_size.pending
			|| self.parked_size.get() < pool_size.parked
			|| self.pending_max_txs_bytes.get() < pool_size.pending_size
			|| self.parked_max_txs_bytes.get() < pool_size.parked_size
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubPoolLimit {
	/// Maximum amount of transaction in the pool.
	pub max_txs: usize,
	/// Maximum combined size (in bytes) of transactions in the pool.
	pub max_size: usize,
}

impl SubPoolLimit {
	/// Creates a new instance with the given limits.
	pub const fn new(max_txs: usize, max_size: usize) -> Self {
		Self { max_txs, max_size }
	}

	/// Returns whether the size or amount constraint is violated.
	#[inline]
	pub const fn is_exceeded(&self, txs: usize, size: usize) -> bool {
		self.max_txs < txs || self.max_size < size
	}
}

impl Mul<usize> for SubPoolLimit {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		let Self { max_txs, max_size } = self;
		Self { max_txs: max_txs * rhs, max_size: max_size * rhs }
	}
}

impl Default for SubPoolLimit {
	fn default() -> Self {
		// either 10k transactions or 20MB
		Self {
			max_txs: TXPOOL_SUBPOOL_MAX_TXS_DEFAULT,
			max_size: TXPOOL_SUBPOOL_MAX_SIZE_MB_DEFAULT * 1024 * 1024,
		}
	}
}

/// Configuration options for the locally received transactions:
/// [`TransactionOrigin::Local`](TransactionOrigin)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalTransactionConfig {
	/// Apply no exemptions to the locally received transactions.
	///
	/// This includes:
	///   - available slots are limited to the configured `max_account_slots` of [`PoolConfig`]
	///   - no price exemptions
	///   - no eviction exemptions
	pub no_exemptions: bool,
	/// Addresses that will be considered as local. Above exemptions apply.
	pub local_addresses: HashSet<Address>,
	/// Flag indicating whether local transactions should be propagated.
	pub propagate_local_transactions: bool,
}

impl Default for LocalTransactionConfig {
	fn default() -> Self {
		Self {
			no_exemptions: false,
			local_addresses: HashSet::default(),
			propagate_local_transactions: true,
		}
	}
}

impl LocalTransactionConfig {
	/// Returns whether local transactions are not exempt from the configured limits.
	#[inline]
	pub const fn no_local_exemptions(&self) -> bool {
		self.no_exemptions
	}

	/// Returns whether the local addresses vector contains the given address.
	#[inline]
	pub fn contains_local_address(&self, address: &Address) -> bool {
		self.local_addresses.contains(address)
	}

	/// Returns whether the particular transaction should be considered local.
	///
	/// This always returns false if the local exemptions are disabled.
	#[inline]
	pub fn is_local(&self, origin: TransactionOrigin, sender: &Address) -> bool {
		if self.no_local_exemptions() {
			return false;
		}
		origin.is_local() || self.contains_local_address(sender)
	}

	/// Sets toggle to propagate transactions received locally by this client (e.g
	/// transactions from `eth_sendTransaction` to this nodes' RPC server)
	///
	/// If set to false, only transactions received by network peers (via
	/// p2p) will be marked as propagated in the local transaction pool and returned on a
	/// `GetPooledTransactions` p2p request
	pub const fn set_propagate_local_transactions(mut self, propagate_local_txs: bool) -> Self {
		self.propagate_local_transactions = propagate_local_txs;
		self
	}
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
	/// Max number of transaction in the pending sub-pool
	pub pending_limit: SubPoolLimit,

	/// Max number of transaction in the queued sub-pool
	pub parked_limit: SubPoolLimit,

	/// Max number of executable transaction slots guaranteed per account
	pub max_account_slots: usize,

	/// Minimum base fee required by the protocol.
	pub minimal_protocol_basefee: u128,
	/// The max gas limit for transactions in the pool
	pub gas_limit: u128,
	/// How to handle locally received transactions:
	/// [`TransactionOrigin::Local`](TransactionOrigin).
	pub local_transactions_config: LocalTransactionConfig,
	/// Maximum lifetime for transactions in the pool
	pub max_queued_lifetime: Duration,
}

impl PoolConfig {
	/// Returns whether the size and amount constraints in any sub-pools are exceeded.
	#[inline]
	pub const fn is_exceeded(&self, pool_size: PoolSize) -> bool {
		self.pending_limit.is_exceeded(pool_size.pending, pool_size.pending_size)
			|| self.parked_limit.is_exceeded(pool_size.parked, pool_size.parked_size)
	}
}

impl Default for PoolConfig {
	fn default() -> Self {
		Self {
			pending_limit: Default::default(),
			parked_limit: Default::default(),

			max_account_slots: TXPOOL_MAX_ACCOUNT_SLOTS_PER_SENDER,
			minimal_protocol_basefee: MIN_PROTOCOL_BASE_FEE,
			gas_limit: BLOCK_GAS_LIMIT,
			local_transactions_config: Default::default(),

			max_queued_lifetime: MAX_QUEUED_TRANSACTION_LIFETIME,
		}
	}
}
