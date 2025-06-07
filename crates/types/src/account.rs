use crate::{address::AccAddress, pub_key::PubKey};

/// AccountI defines the interface for accounts in the auth module.
pub trait AccountI<T: PubKey> {
	/// Get the account's address
	fn get_address(&self) -> AccAddress;

	/// Set the account's address. Returns error if already set.
	fn set_address(&mut self, address: AccAddress) -> Result<(), String>;

	/// Get the account's public key. Can return None.
	fn get_pub_key(&self) -> Option<T>;

	/// Set the account's public key
	fn set_pub_key(&mut self, pub_key: T) -> Result<(), String>;

	/// Get the account number
	fn get_account_number(&self) -> u64;

	/// Set the account number
	fn set_account_number(&mut self, account_number: u64) -> Result<(), String>;

	/// Get the account sequence
	fn get_sequence(&self) -> u64;

	/// Set the account sequence
	fn set_sequence(&mut self, sequence: u64) -> Result<(), String>;
}

/// ModuleAccountI defines an account interface for modules that hold tokens in
/// an escrow.
pub trait ModuleAccountI<T: PubKey>: AccountI<T> {
	/// Get the module account name
	fn get_name(&self) -> String;

	/// Get the permissions assigned to the module account
	fn get_permissions(&self) -> Vec<String>;

	/// Check if the module account has a specific permission
	fn has_permission(&self, permission: &str) -> bool;
}
