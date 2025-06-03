use core::error;
use std::error::Error;
use std::fmt;

/// Errors that can occur during codec operations
#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
	#[error("decoding error: {0}")]
	DecodeError(String),
	#[error("encoding error: {0}")]
	EncodeError(String),
	#[error("prefix already taken")]
	PrefixTakenError(String),
	#[error("name already taken")]
	NameTakenError(String),
	#[error("name must match regex {0}")]
	NameRegexError(String),

	#[error("prefixes cannot overlap {0} and {1}")]
	PrefixOverlapError(String, String),

	#[error("conflict error")]
	ConflictError,

	#[error("not found error")]
	NotFoundError,
}
