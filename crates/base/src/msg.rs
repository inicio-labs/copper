use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RoutableMsg {
	module: String,
	msg_id: String,
	content: Bytes,
}

impl RoutableMsg {
	pub fn new(module: String, msg_id: String, content: Bytes) -> Self {
		Self { module, msg_id, content }
	}

	pub fn module(&self) -> &str {
		&self.module
	}

	pub fn msg_id(&self) -> &str {
		&self.msg_id
	}

	pub fn content(&self) -> &Bytes {
		&self.content
	}

	pub fn dissolve(self) -> (String, String, Bytes) {
		let Self { module, msg_id, content } = self;
		(module, msg_id, content)
	}
}
