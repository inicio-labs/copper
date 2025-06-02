use std::{sync::Arc, time::Instant};

use crate::{
	config::MIN_PROTOCOL_BASE_FEE,
	identifier::{SenderIdentifiers, TransactionId},
	ordering::{Priority, TransactionOrdering},
	traits::{PoolTransaction, TransactionOrigin},
	util::{Address, TxHash},
	validate::ValidPoolTransaction,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MockTransaction {
	/// The chain id of the transaction.
	chain_id: Option<String>,
	/// The hash of the transaction.
	hash: TxHash,
	/// The sender's address.
	sender: Address,
	/// The transaction nonce.
	nonce: u64,
	/// The gas price for the transaction.
	gas_price: u128,
	/// The gas limit for the transaction.
	gas_limit: u128,
	/// The transaction input data.
	input: Vec<u8>,
}

impl Default for MockTransaction {
	fn default() -> Self {
		Self {
			chain_id: Some("test-123".to_string()),
			hash: TxHash::new([0; 32]),
			sender: Address::new("0x1234567890123456789012345678901234567890".to_string()),
			nonce: 0,
			gas_price: MIN_PROTOCOL_BASE_FEE,
			gas_limit: 100,
			input: vec![0; 100],
		}
	}
}

impl PoolTransaction for MockTransaction {
	fn hash(&self) -> &TxHash {
		&self.hash
	}

	fn sender(&self) -> Address {
		self.sender.clone()
	}

	fn sender_ref(&self) -> &Address {
		&self.sender
	}

	fn cost(&self) -> u128 {
		self.gas_price * self.gas_limit
	}

	fn encoded_length(&self) -> usize {
		self.input.len()
	}

	fn max_fee_per_gas(&self) -> u128 {
		self.gas_price
	}

	fn gas(&self) -> u128 {
		self.gas_limit
	}

	fn nonce(&self) -> u64 {
		self.nonce
	}

	fn size(&self) -> usize {
		self.input.len()
	}
}

impl MockTransaction {
	/// Sets the chain ID of the transaction.
	pub fn set_chain_id(&mut self, chain_id: Option<String>) -> &mut Self {
		self.chain_id = chain_id;
		self
	}

	/// Sets the hash of the transaction.
	pub fn set_hash(&mut self, hash: TxHash) -> &mut Self {
		self.hash = hash;
		self
	}

	/// Sets the sender's address.
	pub fn set_sender(&mut self, sender: Address) -> &mut Self {
		self.sender = sender;
		self
	}

	/// Sets the transaction nonce.
	pub fn set_nonce(&mut self, nonce: u64) -> &mut Self {
		self.nonce = nonce;
		self
	}

	pub fn inc_nonce(&mut self) -> &mut Self {
		self.nonce += 1;
		self
	}

	/// Sets the gas price for the transaction.
	pub fn set_gas_price(&mut self, gas_price: u128) -> &mut Self {
		self.gas_price = gas_price;
		self
	}

	/// Sets the gas limit for the transaction.
	pub fn set_gas_limit(&mut self, gas_limit: u128) -> &mut Self {
		self.gas_limit = gas_limit;
		self
	}

	/// Sets the transaction input data.
	pub fn set_input(&mut self, input: Vec<u8>) -> &mut Self {
		self.input = input;
		self
	}
}

impl TransactionOrdering for MockTransaction {
	type PriorityValue = u128;
	type Transaction = MockTransaction;

	fn priority(&self, _: &Self::Transaction) -> Priority<Self::PriorityValue> {
		Priority::Value(self.cost())
	}
}
/// This type is an alias for [`ValidPoolTransaction<MockTransaction>`].
pub type MockValidTx = ValidPoolTransaction<MockTransaction>;

/// A factory for creating and managing various types of mock transactions.
#[derive(Debug, Default)]
pub struct MockTransactionFactory {
	pub(crate) ids: SenderIdentifiers,
}

// === impl MockTransactionFactory ===

impl MockTransactionFactory {
	/// Generates a transaction ID for the given [`MockTransaction`].
	pub fn tx_id(&mut self, tx: &MockTransaction) -> TransactionId {
		let sender = self.ids.sender_id_or_create(tx.sender());
		TransactionId::new(sender, tx.nonce())
	}

	/// Validates a [`MockTransaction`] and returns a [`MockValidTx`].
	pub fn validated(&mut self, transaction: MockTransaction) -> MockValidTx {
		self.validated_with_origin(TransactionOrigin::External, transaction)
	}

	/// Validates a [`MockTransaction`] and returns a shared [`Arc<MockValidTx>`].
	pub fn validated_arc(&mut self, transaction: MockTransaction) -> Arc<MockValidTx> {
		Arc::new(self.validated(transaction))
	}

	/// Converts the transaction into a validated transaction with a specified origin.
	pub fn validated_with_origin(
		&mut self,
		origin: TransactionOrigin,
		transaction: MockTransaction,
	) -> MockValidTx {
		MockValidTx {
			propagate: false,
			transaction_id: self.tx_id(&transaction),
			transaction,
			timestamp: Instant::now(),
			origin,
		}
	}

	/// Creates a validated legacy [`MockTransaction`].
	pub fn create_tx(&mut self) -> MockValidTx {
		self.validated(MockTransaction::default())
	}
}

/// A set of mock transactions that can be generated in various configurations.
#[derive(Debug, Clone)]
pub struct MockTransactionSet {
	pub transactions: Vec<MockTransaction>,
}

impl MockTransactionSet {
	/// Creates a set of dependent transactions (sequential nonces) for a given sender.
	///
	/// # Arguments
	/// * `sender` - The address that will be the sender for all transactions
	/// * `start_nonce` - The starting nonce value
	/// * `end_nonce` - The ending nonce value (inclusive)
	pub fn dependent(sender: Address, start_nonce: u64, end_nonce: u64) -> Self {
		let mut transactions = Vec::new();

		for nonce in start_nonce..=end_nonce {
			let mut tx = MockTransaction::default();
			tx.set_sender(sender.clone()).set_nonce(nonce);

			// Create unique hash for each transaction
			let mut hash_bytes = [0u8; 32];
			hash_bytes[0..8].copy_from_slice(&nonce.to_le_bytes());
			tx.set_hash(TxHash::new(hash_bytes));

			transactions.push(tx);
		}

		Self { transactions }
	}

	/// Converts the transaction set into a vector of transactions.
	pub fn into_vec(self) -> Vec<MockTransaction> {
		self.transactions
	}

	/// Returns a reference to the transactions vector.
	pub fn as_vec(&self) -> &Vec<MockTransaction> {
		&self.transactions
	}

	/// Returns the number of transactions in the set.
	pub fn len(&self) -> usize {
		self.transactions.len()
	}

	/// Returns true if the transaction set is empty.
	pub fn is_empty(&self) -> bool {
		self.transactions.is_empty()
	}

	pub fn extend(&mut self, other: Self) {
		self.transactions.extend(other.transactions);
	}
}
