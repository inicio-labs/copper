use copper_types::address::AccAddress;

/// Standard permission types
pub const MINTER: &str = "minter";
pub const BURNER: &str = "burner";
pub const STAKING: &str = "staking";

/// PermissionsForAddress defines all the registered permissions for an address
#[derive(Clone, Debug)]
pub struct PermissionsForAddress {
	permissions: Vec<String>,
	address: AccAddress,
}

impl PermissionsForAddress {
	/// Creates a new PermissionsForAddress object
	pub fn new(name: &str, permissions: Vec<String>) -> Result<Self, String> {
		// Validate permissions before creating the object
		validate_permissions(&permissions)?;

		let address = match AccAddress::from_str(name) {
			Ok(address) => address,
			Err(e) => return Err(e.to_string()),
		};

		Ok(Self {
			permissions,
			address, // Assuming this function exists elsewhere
		})
	}

	/// Returns whether the PermissionsForAddress contains permission
	pub fn has_permission(&self, permission: &str) -> bool {
		self.permissions.iter().any(|perm| perm == permission)
	}

	/// Returns the address of the PermissionsForAddress object
	pub fn get_address(&self) -> &AccAddress {
		&self.address
	}

	/// Returns the permissions granted to the address
	pub fn get_permissions(&self) -> &[String] {
		&self.permissions
	}
}

/// Performs basic permission validation
fn validate_permissions(permissions: &[String]) -> Result<(), String> {
	for perm in permissions {
		if perm.trim().is_empty() {
			return Err("module permission is empty".to_string());
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_permissions_validation() {
		// Test empty permissions
		let empty_perm = vec![" ".to_string()];
		assert!(validate_permissions(&empty_perm).is_err());

		// Test valid permissions
		let valid_perms = vec!["minter".to_string(), "burner".to_string()];
		assert!(validate_permissions(&valid_perms).is_ok());
	}

	#[test]
	fn test_has_permission() {
		let perms =
			PermissionsForAddress::new("test", vec![MINTER.to_string(), BURNER.to_string()])
				.unwrap();

		assert!(perms.has_permission(MINTER));
		assert!(perms.has_permission(BURNER));
		assert!(!perms.has_permission(STAKING));
	}
}
