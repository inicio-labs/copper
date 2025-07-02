/// Queryable allows a Store to expose internal state to the abci.Query
/// interface. Multistore can route requests to the proper Store.
///
/// This is an optional, but useful extension to any CommitStore
pub trait Queryable {
	type Error;

	fn query(&self, req: &RequestQuery) -> Result<ResponseQuery, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RequestQuery {
	pub data: Vec<u8>,
	pub path: String,
	pub height: i64,
	pub prove: bool,
}

#[derive(Debug, Clone)]
pub struct ResponseQuery {
	pub code: u32,
	pub log: String,
	pub info: String,
	pub index: i64,
	pub key: Vec<u8>,
	pub value: Vec<u8>,
	pub proof_ops: Option<Vec<u8>>, // TODO: Replace with proper ProofOps type when available
	pub height: i64,
	pub codespace: String,
}
