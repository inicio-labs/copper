use crate::address::AccAddress;

pub fn new_module_address(name: &str) -> Result<AccAddress, Box<dyn std::error::Error>> {
	// Calculate SHA256 hash of the name
	// TODO: Implement this Sha256

	let hash = name.as_bytes();

	// Convert first 20 bytes of hash to address
	let address = AccAddress::new(&hash[..20])?;

	Ok(address)
}
