use crate::codec::codec::KeyCodec;
use crate::codec::error::CodecError;
use std::marker::PhantomData;

/// Maximum length of a bytes key when encoded
pub const MAX_BYTES_KEY_SIZE: u8 = u8::MAX;

/// BytesCodec implements KeyCodec for byte slice types
pub struct BytesCodec<T>(PhantomData<T>);

impl<T> BytesCodec<T> {
	/// Creates a new BytesCodec instance
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: AsRef<[u8]> + From<Vec<u8>>> KeyCodec<T> for BytesCodec<T> {
	fn encode(&self, buffer: &mut [u8], key: &T) -> Result<usize, CodecError> {
		let key_bytes = key.as_ref();
		buffer
			.get_mut(..key_bytes.len())
			.ok_or_else(|| CodecError::EncodingError("buffer too small".to_string()))?
			.copy_from_slice(key_bytes);
		Ok(key_bytes.len())
	}

	fn decode(&self, buffer: &[u8]) -> Result<(usize, T), CodecError> {
		// Convert the buffer directly to T since we own the data
		Ok((buffer.len(), Vec::from(buffer).into()))
	}

	fn size(&self, key: &T) -> usize {
		key.as_ref().len()
	}

	fn stringify(&self, key: &T) -> String {
		format!("hexBytes:{}", hex::encode(key.as_ref()))
	}

	fn key_type(&self) -> String {
		"bytes".to_string()
	}
}
