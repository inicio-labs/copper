mod error;

pub use self::error::BankKeeperError;

use core::num::NonZeroU128;

use copper_base::{Address, Coin};
use copper_collections::Map;
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use nebz::NonEmptyBz;

use self::error::Result;

pub struct BankKeeper {
	balances: Map<'static, (Address, String), NonZeroU128>,
}

impl BankKeeper {
	pub fn new(prefix: NonEmptyBz<&'static [u8]>) -> Self {
		Self { balances: Map::new(prefix) }
	}

	pub fn balance<S>(
		&self,
		store: &S,
		address_denom: &(Address, String),
	) -> Result<Option<NonZeroU128>>
	where
		S: GetKVStore,
	{
		self.balances.get(store, address_denom).map_err(From::from)
	}

	pub fn send_coin<S>(
		&self,
		store: &mut S,
		from: &Address,
		to: &Address,
		coin: Coin,
	) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
	{
		let from_denom = &(*from, coin.denom().into());
		let from_balance = self.balance(store, from_denom)?.map(NonZeroU128::get).unwrap_or(0);

		let to_denom = &(*to, coin.denom().into());
		let to_balance = self.balance(store, to_denom)?.map(NonZeroU128::get).unwrap_or(0);

		let new_from_balance =
			from_balance.checked_sub(coin.amount()).ok_or(BankKeeperError::InsufficientBalance)?;
		let new_to_balance =
			to_balance.checked_add(coin.amount()).ok_or(BankKeeperError::BalanceOverflow)?;

		self.set_balance(store, from_denom, new_from_balance)?;
		self.set_balance(store, to_denom, new_to_balance)?;

		Ok(())
	}

	pub(crate) fn set_balance<S>(
		&self,
		store: &mut S,
		address_denom: &(Address, String),
		amount: u128,
	) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
		NonEmptyBz<S::Key>: for<'a> From<NonEmptyBz<&'a [u8]>>,
	{
		let amount = match NonZeroU128::new(amount) {
			Some(a) => a,
			None => {
				self.balances.remove(store, address_denom)?;
				return Ok(());
			},
		};

		self.balances.insert(store, address_denom, &amount)?;

		Ok(())
	}
}
