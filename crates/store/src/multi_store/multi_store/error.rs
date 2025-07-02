use std::sync::PoisonError;

// Error types
use thiserror::Error;

use crate::types::store::StoreType;

pub type Result<T, E = StoreError> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum StoreError {
	#[error("Store key cannot be nil")]
	NilKey,
	#[error("Duplicate store key: {0}")]
	DuplicateKey(String),
	#[error("Duplicate store key: {0}")]
	DuplicateKeyName(String),
	#[error("Store does not exist for key: {0}")]
	StoreNotFound(String),
	#[error("Database error: {0}")]
	DatabaseError(String),
	#[error("Version mismatch: expected {expected}, got {actual}")]
	VersionMismatch { expected: i64, actual: i64 },
	#[error("Invalid path: {0}")]
	InvalidPath(String),
	#[error("Store type not supported: {0:?}")]
	UnsupportedStoreType(StoreType),

	#[error("IAVL load error: {0}")]
	IAVLLoadError(String),

	#[error("Poisoned lock")]
	PoisonedLock,
}

impl<T> From<PoisonError<T>> for StoreError {
	fn from(_e: PoisonError<T>) -> Self {
		StoreError::PoisonedLock
	}
}
