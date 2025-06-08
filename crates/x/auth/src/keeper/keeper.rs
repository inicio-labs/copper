use copper_collections::{
	CollectionError,
	codec::{BytesKeyCodec, BytesValueCodec},
	context::Context,
	item::Item,
	map::Map,
	schema::{Schema, SchemaBuilder},
	sequence::Sequence,
	store::KVStore,
};
use copper_types::{account::AccountI, address::AccAddress, pub_key::PubKey};
use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{
	keeper::account::AccountKeeperI,
	types::{
		keys::{ACCOUNT_NUMBER_STORE_KEY_PREFIX, GLOBAL_ACCOUNT_NUMBER_KEY, PARAMS_KEY},
		permissions::PermissionsForAddress,
	},
};

pub struct AccountKeeper<C: Context + Clone + 'static, K: KVStore<C, CollectionError> + Clone> {
	store_service: K,
	perm_addrs: HashMap<String, PermissionsForAddress>,
	authority: String,

	// State
	schema: Schema<C, K>,
	params: Item<Vec<u8>, BytesValueCodec, C>,
	account_number: Sequence<C>,
	accounts: Map<Vec<u8>, Vec<u8>, BytesKeyCodec, BytesValueCodec, C>,
}

impl<C: Context + Clone + 'static, K: KVStore<C, CollectionError> + Clone> AccountKeeper<C, K> {
	pub fn new(
		store_service: K,
		macc_perms: HashMap<String, Vec<String>>,
		authority: String,
	) -> Result<Self, CollectionError> {
		let mut perm_addrs = HashMap::new();
		for (name, perms) in macc_perms {
			perm_addrs.insert(name.clone(), PermissionsForAddress::new(&name, perms)?);
		}

		let mut sb = SchemaBuilder::new(store_service.clone());

		let arc_store_service: Arc<Box<dyn KVStore<C, CollectionError>>> = Arc::new(Box::new(store_service.clone()));

		let params = Item::new(
			&mut sb,
			arc_store_service.clone(),
			PARAMS_KEY.to_vec(),
			"params".to_string(),
			BytesValueCodec,
		)?;	

		let account_number = Sequence::new(
			&mut sb,
			arc_store_service.clone(),
			GLOBAL_ACCOUNT_NUMBER_KEY.to_vec(),
			"account_number".to_string(),
		)?;

		let accounts = Map::new(
			&mut sb,
			arc_store_service.clone(),
			ACCOUNT_NUMBER_STORE_KEY_PREFIX.to_vec(),
			"accounts".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)?;

		let schema = sb.build()?;

		Ok(Self { store_service,perm_addrs, authority, schema, params, account_number, accounts })
	}

	// Implementation of key methods...
	pub fn get_authority(&self) -> &str {
		&self.authority
	}


	// ... Additional method implementations would follow
}

impl<C: Context + Clone + 'static, K: KVStore<C, CollectionError> + Clone, T: PubKey>
	AccountKeeperI<C, T> for AccountKeeper<C, K>
{
	/// Return a new account with the next account number and the specified address.
	/// Does not save the new account to the store.
	fn new_account_with_address(&self, ctx: &C, addr: AccAddress) -> Box<dyn AccountI<T>>{
		let acc = self.new_account(ctx, account);
		self.set_account(ctx, acc);
		acc
	};

	/// Return a new account with the next account number.
	/// Does not save the new account to the store.
	fn new_account(&self, ctx: &C, account: Box<dyn AccountI<T>>) -> Box<dyn AccountI<T>>{

	};

	/// Check if an account exists in the store.
	fn has_account(&self, ctx: &C, addr: &AccAddress) -> bool{
	};

	/// Retrieve an account from the store.
	fn get_account(&self, ctx: &C, addr: &AccAddress) -> Option<Box<dyn AccountI<T>>>;

	/// Set an account in the store.
	fn set_account(&self, ctx: &C, account: Box<dyn AccountI<T>>){
		self.accounts.set(ctx, account.get_address(), account.to_vec())?;
	};

	/// Remove an account from the store.
	fn remove_account(&self, ctx: &C, account: Box<dyn AccountI<T>>);

	/// Iterate over all accounts, calling the provided function.
	/// Stop iteration when it returns true.
	fn iterate_accounts<F>(&self, ctx: &C, f: F)
	where
		F: FnMut(Box<dyn AccountI<T>>) -> bool;

	/// Fetch the public key of an account at a specified address
	fn get_pub_key(&self, ctx: &C, addr: &AccAddress) -> Result<Box<dyn PubKey>, String> {
		let acc = self
			.get_account(ctx, addr)
			.ok_or_else(|| format!("account {} does not exist", addr.to_string()))?;

		





	}

	/// Fetch the sequence of an account at a specified address.
	fn get_sequence(&self, ctx: &C, addr: &AccAddress) -> Result<u64, String>;

	/// Fetch the next account number, and increment the internal counter.
	fn next_account_number(&self, ctx: &C) -> u64 {
		let seq = self.account_number.next(ctx);

		match seq {
			Ok(seq) => seq.try_into(),
			Err(e) => {
				panic!("Error getting next account number: {}", e);
			},
		}
	}

	/// Get module permissions
	fn get_module_permissions(&self) -> &HashMap<String, PermissionsForAddress> {
		&self.perm_addrs
	}
}
