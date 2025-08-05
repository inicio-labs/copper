mod error;

use crate::genesis::Balance;

pub use self::error::BankKeeperError;

use core::num::NonZeroU128;

use copper_base::{
	Address,
	coin::{Coin, Denom},
};
use copper_collections::Map;
use copper_store::{GetKVStore, InsertKVStore, RemoveKVStore};
use nebz::NonEmptyBz;

use self::error::Result;

pub struct BankKeeper<'a> {
	balances: Map<'static, (&'a Address, &'a Denom), NonZeroU128>,
}

impl<'a> BankKeeper<'a> {
	pub fn new(prefix: NonEmptyBz<&'static [u8]>) -> Self {
		Self { balances: Map::new(prefix) }
	}

	pub fn balance<S>(
		&self,
		store: &S,
		address: &Address,
		denom: &Denom,
	) -> Result<Option<NonZeroU128>>
	where
		S: GetKVStore,
	{
		self.balances.get(store, &(address, denom)).map_err(From::from)
	}

	pub fn init_balances<S>(&self, store: &mut S, balances: &[Balance]) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
		NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
	{
		for balance in balances {
			for coin in balance.coins() {
				if coin.amount() > 0 {
					self.set_balance(store, balance.address(), coin.denom(), coin.amount())?;
				}
			}
		}

		Ok(())
	}

	pub fn send_coin<S>(
		&self,
		store: &mut S,
		from: &Address,
		to: &Address,
		coin: &Coin,
	) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
		NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
	{
		let from_balance =
			self.balance(store, from, coin.denom())?.map(NonZeroU128::get).unwrap_or(0);

		let to_balance = self.balance(store, to, coin.denom())?.map(NonZeroU128::get).unwrap_or(0);

		let new_from_balance =
			from_balance.checked_sub(coin.amount()).ok_or(BankKeeperError::InsufficientBalance)?;
		let new_to_balance =
			to_balance.checked_add(coin.amount()).ok_or(BankKeeperError::BalanceOverflow)?;

		self.set_balance(store, from, coin.denom(), new_from_balance)?;
		self.set_balance(store, to, coin.denom(), new_to_balance)?;

		println!(
			"FROM BALANCE AFTER COIN SEND: {:?}",
			self.balance(store, from, coin.denom())
		);

		println!(
			"TO BALANCE AFTER COIN SEND: {:?}",
			self.balance(store, to, coin.denom())
		);

		Ok(())
	}

	pub(crate) fn set_balance<S>(
		&self,
		store: &mut S,
		address: &Address,
		denom: &Denom,
		amount: u128,
	) -> Result<()>
	where
		S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
		NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
	{
		let Some(amount) = NonZeroU128::new(amount) else {
			self.balances.remove(store, &(address, denom))?;

			return Ok(());
		};

		self.balances.insert(store, &(address, denom), &amount)?;

		Ok(())
	}
}
