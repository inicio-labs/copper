#[derive(Debug, thiserror::Error)]
pub enum CollectionsError {
	#[error("serialization error")]
	Serialization,

	#[error("deserialization error")]
	Deserialization,

	#[error("store error")]
	Store,
}
