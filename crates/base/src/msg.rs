use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

use crate::Address;

pub trait Msg {
	fn signers(&self) -> Vec<Address>;
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RoutableMsg {
	route: Bytes,
	content: Bytes,
}
