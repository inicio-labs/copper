use hex::encode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccAddress([u8; 20]);

impl AccAddress {
	pub fn new(given_address: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
		if given_address.len() != 20 {
			return Err(format!("Invalid address length: {}", given_address.len()).into());
		}

		let mut address = [0u8; 20];
		address.copy_from_slice(given_address);

		Ok(Self(address))
	}

	pub fn to_string(&self) -> String {
		hex::encode(self.0)
	}

	pub fn from_str(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let mut address = [0u8; 20];
		hex::decode_to_slice(address, &mut address)?;

		Ok(Self(address))
	}
}

impl From<[u8; 20]> for AccAddress {
	fn from(address: [u8; 20]) -> Self {
		Self(address)
	}
}
