use copper_proto::cosmos::auth::v1beta1::BaseAccount as BaseAccountProto;
use copper_types::{address::AccAddress, pub_key::PubKey};

pub struct BaseAccount {
	pub inner: BaseAccountProto,
}

impl BaseAccount {
	pub fn new<P: PubKey>(
		address: AccAddress,
		pub_key: P,
		account_number: u64,
		sequence: u64,
	) -> Self {
		Self {
			inner: BaseAccountProto {
				address: address.to_string(),
				pub_key: Some(pub_key.to_any()),
				account_number,
				sequence,
			},
		}
	}

	pub fn set_address(&mut self, address: AccAddress) {
		self.inner.address = address.to_string();
	}

	pub fn get_address(&self) -> AccAddress {
		AccAddress::from_str(&self.inner.address).unwrap()
	}

	pub fn set_pub_key<P: PubKey>(&mut self, pub_key: P) {
		self.inner.pub_key = Some(pub_key.to_any());
	}

	// TODO: Implement this
	// pub fn get_pub_key(&self) -> Option<Box<dyn PubKey>> {
	// 	self.inner.pub_key.map(|any| Box::new(PubKey::new(any)))
	// }

	pub fn get_account_number(&self) -> u64 {
		self.inner.account_number
	}

	pub fn get_sequence(&self) -> u64 {
		self.inner.sequence
	}
}
