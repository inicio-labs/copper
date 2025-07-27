#[derive(Debug, thiserror::Error)]
pub enum DenomError {
	#[error("minimum length error")]
	MinimumLength,

	#[error("invalid char error")]
	InvalidChar,
}
