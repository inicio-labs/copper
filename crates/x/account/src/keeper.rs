mod error;

pub use self::error::AccountKeeperError;

use bytes::Bytes;
use copper_base::Address;
use copper_collections::Map;
use copper_store::{GetKVStore, InsertKVStore};
use nebz::NonEmptyBz;

use crate::types::BaseAccount;

use self::error::Result;

pub struct AccountKeeper {
	accounts: Map<'static, Address, BaseAccount>,
}

impl AccountKeeper {
	pub fn new(prefix: NonEmptyBz<&'static [u8]>) -> Self {
		Self { accounts: Map::new(prefix) }
	}

	pub fn get_account<S>(&self, store: &S, address: &Address) -> Result<Option<BaseAccount>>
	where
		S: GetKVStore,
	{
		self.accounts.get(store, address).map_err(From::from)
	}

	pub fn increment_sequence<S>(&self, store: &mut S, address: &Address) -> Result<u64>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>>,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
	{
		let acc = self.get_account(store, address)?.ok_or(AccountKeeperError::AccountNotFound)?;

		let (pk, sequence) = acc.dissolve();

		let new_acc = sequence
			.checked_add(1)
			.map(|s| BaseAccount::new(pk, s))
			.ok_or(AccountKeeperError::SequenceOverflow)?;

		assert!(self.accounts.insert(store, address, &new_acc)?);

		Ok(new_acc.sequence())
	}

	pub fn set_pub_key<S>(&self, store: &mut S, address: &Address, pub_key: Bytes) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>>,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
	{
		let new_acc = match self.get_account(store, address)? {
			Some(acc) => BaseAccount::new(pub_key, acc.sequence()),
			None => BaseAccount::new(pub_key, 0),
		};

		assert!(self.accounts.insert(store, address, &new_acc)?);

		Ok(())
	}
}
