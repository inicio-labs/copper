mod best;
mod config;
mod error;
mod identifier;
mod mempool;
mod mined;
mod ordering;
mod parked;
mod pending;
mod size;
mod state;
mod traits;
mod update;
mod util;
mod validate;

#[cfg(test)]
mod testutils;

/// Mempool defines the interface for managing the application's transaction mempool.
pub trait Mempool {
	// /// CheckTx executes a new transaction against the application to determine
	// /// its validity and whether it should be added to the mempool.
	// fn check_tx(
	// 	&self,
	// 	tx: Tx,
	// 	callback: Box<dyn Fn(&abci::ResponseCheckTx)>,
	// 	tx_info: TxInfo,
	// ) -> Result<(), Error>;

	// /// RemoveTxByKey removes a transaction, identified by its key, from the mempool.
	// fn remove_tx_by_key(&mut self, tx_key: TxKey) -> Result<(), Error>;

	// /// Reaps transactions from the mempool up to max_bytes bytes total with the
	// /// condition that the total gas_wanted must be less than max_gas.
	// ///
	// /// If both maxes are negative, there is no cap on the size of all returned
	// /// transactions (~ all available transactions).
	// fn reap_max_bytes_max_gas(&self, max_bytes: i64, max_gas: i64) -> Vec<Tx>;

	// /// Reaps up to max transactions from the mempool. If max is negative, there
	// /// is no cap on the size of all returned transactions.
	// fn reap_max_txs(&self, max: i32) -> Vec<Tx>;

	// /// Locks the mempool. The consensus must be able to hold lock to safely update.
	// /// Before acquiring the lock, it signals the mempool that a new update is coming.
	// /// If the mempool is still rechecking at this point, it should be considered full.
	// fn lock(&mut self);

	// /// Unlocks the mempool.
	// fn unlock(&mut self);

	// /// Update informs the mempool that the given txs were committed and can be discarded.
	// ///
	// /// NOTE:
	// /// 1. This should be called *after* block is committed by consensus.
	// /// 2. Lock/Unlock must be managed by the caller.
	// fn update(
	// 	&mut self,
	// 	block_height: i64,
	// 	block_txs: Vec<Tx>,
	// 	deliver_tx_responses: Vec<abci::ExecTxResult>,
	// 	new_pre_fn: PreCheckFunc,
	// 	new_post_fn: PostCheckFunc,
	// ) -> Result<(), Error>;

	// /// Flushes the mempool connection to ensure async callback calls are done.
	// ///
	// /// NOTE: Lock/Unlock must be managed by caller.
	// fn flush_app_conn(&self) -> Result<(), Error>;

	// /// Removes all transactions from the mempool and caches.
	// fn flush(&mut self);

	// /// Returns a channel which fires once for every height, and only when
	// /// transactions are available in the mempool.
	// ///
	// /// NOTE: The returned channel may be None if enable_txs_available was not called.
	// fn txs_available(&self) -> Option<crossbeam::channel::Receiver<()>>;

	// /// Initializes the TxsAvailable channel, ensuring it will trigger once
	// /// every height when transactions are available.
	// fn enable_txs_available(&mut self);

	// /// Returns the number of transactions in the mempool.
	// fn size(&self) -> usize;

	// /// Returns the total size of all txs in the mempool.
	// fn size_bytes(&self) -> i64;
}
