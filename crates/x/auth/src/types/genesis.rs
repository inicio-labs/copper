use copper_proto::cosmos::auth::v1beta1::{GenesisState as GenesisStateProto, Params};

pub struct GenesisState {
	pub inner: GenesisStateProto,
}

impl GenesisState {
	pub fn new(params: Params, accounts: Vec<prost_types::Any>) -> Self {
		Self { inner: GenesisStateProto { params: Some(params), accounts } }
	}
}
