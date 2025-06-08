use copper_collections::{
	CollectionError,
	codec::{BytesKeyCodec, BytesValueCodec},
	context::Context,
	item::Item,
	map::Map,
	ranger::{Direction, Ranger},
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

pub struct AccountKeeper<
	C: Context + Clone + 'static,
	KV: KVStore<C, CollectionError> + Clone + 'static,
> {
	store_service: KV,
	perm_addrs: HashMap<String, PermissionsForAddress>,
	pub authority: String,

	// State
	schema: Schema<C, KV>,
	pub params: Item<Vec<u8>, BytesValueCodec, C, KV>,
	pub account_number: Sequence<C, KV>,
	pub accounts: Map<Vec<u8>, Vec<u8>, BytesKeyCodec, BytesValueCodec, C, KV>,
}

impl<C: Context + Clone + 'static, KV: KVStore<C, CollectionError> + Clone + 'static>
	AccountKeeper<C, KV>
{
	pub fn new(
		store_service: KV,
		macc_perms: HashMap<String, Vec<String>>,
		authority: String,
	) -> Result<Self, CollectionError> {
		let mut perm_addrs = HashMap::new();
		for (name, perms) in macc_perms {
			perm_addrs.insert(
				name.clone(),
				PermissionsForAddress::new(&name, perms)
					.map_err(|e| CollectionError::InvalidInput(e))?,
			);
		}

		let mut sb = SchemaBuilder::new(store_service.clone());

		let store_accessor = Arc::new(store_service.clone());

		let params = Item::new(
			&mut sb,
			store_accessor.clone(),
			PARAMS_KEY.to_vec(),
			"params".to_string(),
			BytesValueCodec,
		)?;

		let account_number = Sequence::new(
			&mut sb,
			store_accessor.clone(),
			GLOBAL_ACCOUNT_NUMBER_KEY.to_vec(),
			"account_number".to_string(),
		)?;

		let accounts = Map::new(
			&mut sb,
			store_accessor.clone(),
			ACCOUNT_NUMBER_STORE_KEY_PREFIX.to_vec(),
			"accounts".to_string(),
			BytesKeyCodec,
			BytesValueCodec,
		)?;

		let schema = sb.build()?;

		Ok(Self { store_service, perm_addrs, authority, schema, params, account_number, accounts })
	}

	// Implementation of key methods...
	pub fn get_authority(&self) -> &str {
		&self.authority
	}
}

impl<C, KV, PK, AC> AccountKeeperI<C, PK, AC> for AccountKeeper<C, KV>
where
	C: Context + Clone + 'static,
	KV: KVStore<C, CollectionError> + Clone + 'static,
	PK: PubKey,
	AC: AccountI<PK>,
{
	/// Return a new account with the next account number and the specified address.
	/// Does not save the new account to the store.
	fn new_account_with_address(&self, ctx: &C, addr: AccAddress) -> AC {
		let acc = AC::new(addr);

		acc
	}

	/// Return a new account with the next account number.
	/// Does not save the new account to the store.
	fn new_account(&self, ctx: &C, account: &mut AC) -> Result<(), String> {
		let next_account_number =
			<AccountKeeper<C, KV> as AccountKeeperI<C, PK, AC>>::next_account_number(&self, ctx);

		account.set_sequence(next_account_number);

		Ok(())
	}

	/// Check if an account exists in the store.
	fn has_account(&self, ctx: &C, addr: &AccAddress) -> bool {
		let has = self.accounts.has(ctx, &addr.into());

		match has {
			Ok(has) => has,
			Err(e) => false,
		}
	}

	/// Retrieve an account from the store.
	fn get_account(&self, ctx: &C, addr: &AccAddress) -> Result<AC, String> {
		//let acc_address_bytes = addr.to_vec();
		let acc_bytes = self.accounts.get(ctx, &addr.into());

		match acc_bytes {
			Ok(acc_bytes) => Ok(AC::from_vec(acc_bytes)),
			Err(e) => Err(e.to_string()),
		}
	}

	/// Set an account in the store.
	fn set_account(&self, ctx: &C, account: &AC) {
		let acc_bytes = account.to_bytes();

		let _ = self.accounts.set(ctx, &account.get_address().into(), &acc_bytes);
	}

	/// Remove an account from the store.
	fn remove_account(&self, ctx: &C, account: &AC) {
		let _ = self.accounts.remove(ctx, &account.get_address().into());
	}

	/// Iterate over all accounts, calling the provided function.
	/// Stop iteration when it returns true.
	fn iterate_accounts<F>(&self, ctx: &C, mut f: F)
	where
		F: FnMut(AC) -> bool,
	{
		let iter = self.accounts.iter(
			ctx,
			Ranger { start: None, end: None, direction: Direction::Asc },
		);

		match iter {
			Ok(iter) => {
				for (_, acc_bytes) in iter {
					let acc = AC::from_vec(acc_bytes);
					if f(acc) {
						break;
					}
				}
			},
			Err(e) => {
				println!("Error iterating over accounts: {}", e);
			},
		}
	}

	/// Fetch the public key of an account at a specified address
	fn get_pub_key(&self, ctx: &C, addr: &AccAddress) -> Result<PK, String> {
		let acc =
			<AccountKeeper<C, KV> as AccountKeeperI<C, PK, AC>>::get_account(&self, ctx, addr)?;
		let pub_key = acc.get_pub_key();

		match pub_key {
			Some(pub_key) => Ok(pub_key),
			None => Err(format!(
				"account {} does not have a public key",
				addr.to_string()
			)),
		}
	}

	/// Fetch the sequence of an account at a specified address.
	fn get_sequence(&self, ctx: &C, addr: &AccAddress) -> Result<u64, String> {
		let acc =
			<AccountKeeper<C, KV> as AccountKeeperI<C, PK, AC>>::get_account(&self, ctx, addr)?;

		Ok(acc.get_sequence())
	}

	/// Fetch the next account number, and increment the internal counter.
	fn next_account_number(&self, ctx: &C) -> u64 {
		let seq = self.account_number.next(ctx);

		match seq {
			Ok(seq) => seq,
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
