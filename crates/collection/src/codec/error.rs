use core::error;
use std::error::Error;
use std::fmt;

/// Errors that can occur during codec operations
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
	#[error("decoding error: {0}")]
	DecodingError(String),
	#[error("encoding error: {0}")]
	EncodingError(String),
}

impl fmt::Display for CodecError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CodecError::DecodingError(msg) => write!(f, "decoding error: {}", msg),
			CodecError::EncodingError(msg) => write!(f, "encoding error: {}", msg),
		}
	}
}

impl Error for CodecError {}
