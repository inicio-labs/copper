use crate::{codec::KeyCodec, CollectionError};

/// NoKeyCodec implements KeyCodec for types that don't have a key
///
#[derive(Debug, Clone, Copy)]
pub struct NoKeyCodec;

impl KeyCodec<i32> for NoKeyCodec {
	fn encode(&self, buffer: &mut Vec<u8>, key: &i32) -> Result<usize, CollectionError> {
		Ok(0)
	}

	fn decode(&self, buffer: &Vec<u8>) -> Result<(usize, i32), CollectionError> {
		Ok((0, 0))
	}

	fn stringify(&self, key: &i32) -> String {
		format!("no_key")
	}

	fn key_type(&self) -> String {
		"no_key".to_string()
	}

	fn size(&self, key: &i32) -> usize {
		0
	}
}
