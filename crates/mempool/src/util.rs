use core::fmt;
use std::fmt::Display;

use crate::traits::PoolTransaction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
	pub fn new(address: String) -> Self {
		Self(address)
	}

	pub fn to_string(&self) -> String {
		self.0.clone()
	}
}

impl Display for Address {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
	pub fn new(hash: [u8; 32]) -> Self {
		Self(hash)
	}

	pub fn to_string(&self) -> String {
		hex::encode(self.0)
	}
}

impl Default for Hash {
	fn default() -> Self {
		Self([0; 32])
	}
}

impl Display for Hash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub struct TxHash([u8; 32]);

impl TxHash {
	pub fn new(hash: [u8; 32]) -> Self {
		Self(hash)
	}

	pub fn to_string(&self) -> String {
		hex::encode(self.0)
	}
}

impl Display for TxHash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Transaction {
	pub from: Address,
	pub gas: u128,
	pub gas_price: u128,
	pub nonce: u64,
	pub inner: Vec<u8>,
}

impl Transaction {
	pub fn new(from: Address, gas: u128, gas_price: u128, nonce: u64, inner: Vec<u8>) -> Self {
		Self { from, gas, gas_price, nonce, inner }
	}

	pub fn sender(&self) -> Address {
		self.from.clone()
	}

	pub fn gas_price(&self) -> u128 {
		self.gas_price
	}

	pub fn cost(&self) -> u128 {
		self.gas * self.gas_price
	}

	pub fn encoded_length(&self) -> usize {
		self.inner.len()
	}
}

impl PoolTransaction for Transaction {
	fn hash(&self) -> &TxHash {
		todo!()
	}

	fn sender(&self) -> Address {
		self.from.clone()
	}

	fn sender_ref(&self) -> &Address {
		&self.from
	}

	fn cost(&self) -> u128 {
		self.gas * self.gas_price
	}

	fn encoded_length(&self) -> usize {
		self.inner.len()
	}

	fn max_fee_per_gas(&self) -> u128 {
		self.gas_price as u128
	}

	fn gas(&self) -> u128 {
		self.gas
	}

	fn nonce(&self) -> u64 {
		self.nonce
	}

	fn size(&self) -> usize {
		self.inner.len()
	}
}

impl Display for Transaction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}
