use copper_proto::cosmos::auth::v1beta1::Params as ParamsProto;

// Default parameter values
const DEFAULT_MAX_MEMO_CHARACTERS: u64 = 256;
const DEFAULT_TX_SIG_LIMIT: u64 = 7;
const DEFAULT_TX_SIZE_COST_PER_BYTE: u64 = 10;
const DEFAULT_SIG_VERIFY_COST_ED25519: u64 = 590;
const DEFAULT_SIG_VERIFY_COST_SECP256K1: u64 = 1000;

pub struct Params {
	inner: ParamsProto,
}

impl Params {
	/// Creates a new Params object with the specified values
	pub fn new(
		max_memo_characters: u64,
		tx_sig_limit: u64,
		tx_size_cost_per_byte: u64,
		sig_verify_cost_ed25519: u64,
		sig_verify_cost_secp256k1: u64,
	) -> Self {
		Self {
			inner: ParamsProto {
				max_memo_characters,
				tx_sig_limit,
				tx_size_cost_per_byte,
				sig_verify_cost_ed25519,
				sig_verify_cost_secp256k1,
			},
		}
	}

	/// Validates that the parameters have valid values
	pub fn validate(&self) -> Result<(), String> {
		if self.inner.tx_sig_limit == 0 {
			return Err("invalid tx signature limit: 0".to_string());
		}
		if self.inner.sig_verify_cost_ed25519 == 0 {
			return Err("invalid ED25519 signature verification cost: 0".to_string());
		}
		if self.inner.sig_verify_cost_secp256k1 == 0 {
			return Err("invalid SECK256k1 signature verification cost: 0".to_string());
		}
		if self.inner.max_memo_characters == 0 {
			return Err("invalid max memo characters: 0".to_string());
		}
		if self.inner.tx_size_cost_per_byte == 0 {
			return Err("invalid tx size cost per byte: 0".to_string());
		}

		Ok(())
	}
}

impl Default for Params {
	fn default() -> Self {
		Self {
			inner: ParamsProto {
				max_memo_characters: DEFAULT_MAX_MEMO_CHARACTERS,
				tx_sig_limit: DEFAULT_TX_SIG_LIMIT,
				tx_size_cost_per_byte: DEFAULT_TX_SIZE_COST_PER_BYTE,
				sig_verify_cost_ed25519: DEFAULT_SIG_VERIFY_COST_ED25519,
				sig_verify_cost_secp256k1: DEFAULT_SIG_VERIFY_COST_SECP256K1,
			},
		}
	}
}
