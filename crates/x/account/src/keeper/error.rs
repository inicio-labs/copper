use copper_collections::CollectionsError;

pub type Result<T, E = AccountKeeperError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum AccountKeeperError {
	#[error("collections error")]
	Collections(#[from] CollectionsError),

	#[error("account not found error")]
	AccountNotFound,

	#[error("sequence overflow error")]
	SequenceOverflow,
}
