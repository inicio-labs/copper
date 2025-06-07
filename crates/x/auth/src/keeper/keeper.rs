use copper_collections::{context::Context, schema::Schema, store::KVStore};
use copper_crypto::PubKey;
use copper_proto::Message;
use copper_store::KVStore;
use copper_types::{AccAddress, Codec};
use std::{collections::HashMap, error::Error};

use crate::types::permissions::PermissionsForAddress;

/// AccountKeeper encodes/decodes accounts using binary encoding/decoding
pub struct AccountKeeper<C: Context, E: Error, K: KVStore<C, E>> {
	store_service: K,
	perm_addrs: HashMap<String, PermissionsForAddress>,
	authority: String,

	// State
	schema: Schema<C, K, E>,
	params: Item<Params>,
	account_number: Sequence,
	accounts: IndexedMap<AccAddress, Box<dyn AccountI>, AccountsIndexes>,
}

impl AccountKeeper {
	pub fn new(
		store_service: KVStore,
		proto: Box<dyn Fn() -> Box<dyn AccountI>>,
		macc_perms: HashMap<String, Vec<String>>,
		address_codec: Codec,
		bech32_prefix: String,
		authority: String,
	) -> Result<Self, String> {
		let mut perm_addrs = HashMap::new();
		for (name, perms) in macc_perms {
			perm_addrs.insert(name.clone(), PermissionsForAddress::new(&name, perms)?);
		}

		let mut sb = SchemaBuilder::new(store_service.clone());

		let params = Item::new(&mut sb, PARAMS_KEY, "params");
		let account_number = Sequence::new(&mut sb, GLOBAL_ACCOUNT_NUMBER_KEY, "account_number");
		let accounts = IndexedMap::new(
			&mut sb,
			ADDRESS_STORE_KEY_PREFIX,
			"accounts",
			AccountsIndexes::new(&mut sb),
		);

		let schema = sb.build().map_err(|e| e.to_string())?;

		Ok(Self {
			address_codec,
			store_service,
			perm_addrs,
			bech32_prefix,
			proto,
			authority,
			schema,
			params,
			account_number,
			accounts,
		})
	}

	// Implementation of key methods...
	pub fn get_authority(&self) -> &str {
		&self.authority
	}

	pub fn get_pub_key(&self, ctx: &Context, addr: &AccAddress) -> Result<Box<dyn PubKey>, String> {
		let acc = self
			.get_account(ctx, addr)
			.ok_or_else(|| format!("account {} does not exist", addr))?;

		Ok(
			acc.get_pub_key()
				.ok_or_else(|| format!("public key for account {} not found", addr))?,
		)
	}

	// ... Additional method implementations would follow
}
