pub use proto::*;

mod proto {
	include!(concat!(env!("OUT_DIR"), "/proto_gen/mod.rs"));

	include!(concat!(env!("OUT_DIR"), "/generated_signer_impls.rs"));

	include!(concat!(
		env!("OUT_DIR"),
		"/generated_extract_signers_from_any_msg.rs"
	));

	trait GetSigners {
		fn signers(&self) -> Vec<&str>;
	}
}
