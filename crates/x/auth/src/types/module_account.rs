use copper_proto::cosmos::auth::v1beta1::ModuleAccount as ModuleAccountProto;
use copper_types::module::new_module_address;

use crate::types::base_account::BaseAccount;

pub struct ModuleAccount {
	inner: ModuleAccountProto,
}

impl ModuleAccount {
	pub fn new(base_account: BaseAccount, name: String, permissions: Vec<String>) -> Self {
		Self {
			inner: ModuleAccountProto { base_account: Some(base_account.inner), name, permissions },
		}
	}

	pub fn get_name(&self) -> String {
		self.inner.name.clone()
	}

	pub fn get_permissions(&self) -> Vec<String> {
		self.inner.permissions.clone()
	}

	pub fn validate(&self) -> Result<(), String> {
		if self.inner.base_account.is_none() {
			return Err("base_account is required".to_string());
		}

		if self.inner.name.trim().is_empty() {
			return Err("name is required".to_string());
		}

		let _ = match new_module_address(&self.inner.name) {
			Ok(_) => return Ok(()),
			Err(e) => return Err(e.to_string()),
		};
	}
}
