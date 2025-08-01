#[derive(Debug, thiserror::Error)]
pub enum SignatureVerificationError {
	#[error("missing init part error")]
	MissingInitPart,

	#[error("missing fin part error")]
	MissingFinPart,

	#[error("proposer not found error")]
	ProposerNotFound,

	#[error("invalid signature error")]
	InvalidSignature,
}
