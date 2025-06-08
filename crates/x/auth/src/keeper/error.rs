use copper_collections::CollectionError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("collection error: {0}")]
	Collection(#[from] CollectionError),

	#[error("proto decode error: {0}")]
	ProtoDecode(#[from] prost::DecodeError),

	#[error("invalid authority")]
	InvalidAuthority,

	#[error("invalid params")]
	InvalidParams(String),

	#[error("missing field")]
	FieldMissing,
}
