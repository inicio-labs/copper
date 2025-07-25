use copper_collections::CollectionsError;

pub type Result<T, E = BankKeeperError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum BankKeeperError {
	#[error("collections error: {0}")]
	Collections(#[from] CollectionsError),

	#[error("insufficient balance error")]
	InsufficientBalance,

	#[error("balance overflow error")]
	BalanceOverflow,
}
