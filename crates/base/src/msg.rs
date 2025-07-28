use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

pub trait Msg {
	fn to_routable_msg(&self) -> RoutableMsg;
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RoutableMsg {
	facet: String,
	msg_id: String,
	content: Bytes,
}

impl RoutableMsg {
	pub fn new(facet: String, msg_id: String, content: Bytes) -> Self {
		Self { facet, msg_id, content }
	}

	pub fn facet(&self) -> &str {
		&self.facet
	}

	pub fn msg_id(&self) -> &str {
		&self.msg_id
	}

	pub fn content(&self) -> &Bytes {
		&self.content
	}

	pub fn dissolve(self) -> (String, String, Bytes) {
		let Self { facet: module, msg_id, content } = self;
		(module, msg_id, content)
	}
}
