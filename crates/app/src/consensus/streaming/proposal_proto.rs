use core::option::Option;

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProposalPart {
	#[prost(oneof = "proposal_part::Part", tags = "1, 2, 3")]
	pub part: Option<proposal_part::Part>,
}

pub mod proposal_part {
	use malachitebft_test::proto;

	#[derive(Clone, PartialEq, prost::Oneof)]
	pub enum Part {
		#[prost(message, tag = "1")]
		Init(proto::ProposalInit),

		#[prost(message, tag = "2")]
		Data(proto::Value),

		#[prost(message, tag = "3")]
		Fin(proto::ProposalFin),
	}
}

impl prost::Name for ProposalPart {
	const NAME: &'static str = "ProposalPart";

	const PACKAGE: &'static str = "test";

	fn full_name() -> prost::alloc::string::String {
		"test.ProposalPart".into()
	}

	fn type_url() -> prost::alloc::string::String {
		"/test.ProposalPart".into()
	}
}
