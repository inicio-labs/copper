use copper_collections::context::Context;
use std::collections::HashMap;

use copper_types::{account::AccountI, address::AccAddress, pub_key::PubKey};

use crate::types::permissions::PermissionsForAddress;

/// AccountKeeperI is the interface contract that x/auth's keeper implements
pub trait AccountKeeperI<C: Context + Clone + 'static, T: PubKey> {
	/// Return a new account with the next account number and the specified address.
	/// Does not save the new account to the store.
	fn new_account_with_address(&self, ctx: &C, addr: AccAddress) -> Box<dyn AccountI<T>>;

	/// Return a new account with the next account number.
	/// Does not save the new account to the store.
	fn new_account(&self, ctx: &C, account: Box<dyn AccountI<T>>) -> Box<dyn AccountI<T>>;

	/// Check if an account exists in the store.
	fn has_account(&self, ctx: &C, addr: &AccAddress) -> bool;

	/// Retrieve an account from the store.
	fn get_account(&self, ctx: &C, addr: &AccAddress) -> Option<Box<dyn AccountI<T>>>;

	/// Set an account in the store.
	fn set_account(&self, ctx: &C, account: Box<dyn AccountI<T>>);

	/// Remove an account from the store.
	fn remove_account(&self, ctx: &C, account: Box<dyn AccountI<T>>);

	/// Iterate over all accounts, calling the provided function.
	/// Stop iteration when it returns true.
	fn iterate_accounts<F>(&self, ctx: &C, f: F)
	where
		F: FnMut(Box<dyn AccountI<T>>) -> bool;

	/// Fetch the public key of an account at a specified address
	fn get_pub_key(&self, ctx: &C, addr: &AccAddress) -> Result<Box<dyn PubKey>, String>;

	/// Fetch the sequence of an account at a specified address.
	fn get_sequence(&self, ctx: &C, addr: &AccAddress) -> Result<u64, String>;

	/// Fetch the next account number, and increment the internal counter.
	fn next_account_number(&self, ctx: &C) -> u64;

	/// Get module permissions
	fn get_module_permissions(&self) -> &HashMap<String, PermissionsForAddress>;
}
