use crate::address::AccAddress;
use prost_types::Any;

/// PubKey defines a public key interface that supports different public key types
pub trait PubKey {
	/// Returns the address derived from the public key
	fn address(&self) -> AccAddress;

	/// Returns the raw bytes of the public key
	fn bytes(&self) -> Vec<u8>;

	/// Verifies a signature over a message
	/// msg: message to verify
	/// sig: signature to verify
	fn verify_signature(&self, msg: &[u8], sig: &[u8]) -> bool;

	/// Checks if this public key equals another public key
	fn equals(&self, other: Box<dyn PubKey>) -> bool;

	/// Returns the type identifier of the public key
	fn type_str(&self) -> String;

	fn to_any(&self) -> Any;
}
