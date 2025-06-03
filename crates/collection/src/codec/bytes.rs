use crate::{
	codec::codec::{KeyCodec, ValueCodec},
	error::CollectionError,
};

/// Maximum length of a bytes key when encoded
pub const MAX_BYTES_KEY_SIZE: u8 = u8::MAX;

/// BytesCodec implements KeyCodec for byte slice types
///
#[derive(Debug, Clone, Copy)]
pub struct BytesKeyCodec;

impl KeyCodec<Vec<u8>> for BytesKeyCodec {
	fn encode(&self, buffer: &mut Vec<u8>, key: &Vec<u8>) -> Result<usize, CollectionError> {
		if key.len() > MAX_BYTES_KEY_SIZE as usize {
			return Err(CollectionError::EncodeError("key too long".to_string()));
		}

		buffer.extend(key);

		Ok(buffer.len())
	}

	fn decode(&self, buffer: &Vec<u8>) -> Result<(usize, Vec<u8>), CollectionError> {
		let mut res: Vec<u8> = vec![];

		res.extend(buffer);

		Ok((res.len(), res))
	}

	fn stringify(&self, key: &Vec<u8>) -> String {
		format!("hexBytes:{}", hex::encode(key.as_slice()))
	}

	fn key_type(&self) -> String {
		"bytes".to_string()
	}

	fn size(&self, key: &Vec<u8>) -> usize {
		key.len()
	}
}

#[derive(Debug, Clone, Copy)]

pub struct BytesValueCodec;

impl ValueCodec<Vec<u8>> for BytesValueCodec {
	fn encode(&self, value: &Vec<u8>) -> Result<Vec<u8>, CollectionError> {
		Ok(value.clone())
	}

	fn decode(&self, bytes: &Vec<u8>) -> Result<Vec<u8>, CollectionError> {
		Ok(bytes.clone())
	}

	fn stringify(&self, value: &Vec<u8>) -> String {
		format!("hexBytes:{}", hex::encode(value.as_slice()))
	}

	fn value_type(&self) -> String {
		"bytes".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bytes_codec() {
		let codec = BytesKeyCodec;
		let test_key = vec![1, 2, 3, 4, 5];

		// Test size calculation
		let size = codec.size(&test_key);
		assert_eq!(size, test_key.len());

		// Test encoding
		let mut buffer = Vec::new();
		let written = codec.encode(&mut buffer, &test_key).unwrap();
		assert_eq!(written, size, "written bytes should match buffer size");
		assert_eq!(buffer, test_key, "encoded bytes should match original key");

		// Test decoding
		let (read, decoded_key) = codec.decode(&buffer).unwrap();
		assert_eq!(read, size, "read bytes should match buffer size");
		assert_eq!(
			decoded_key, test_key,
			"decoded key should match original key"
		);

		// Test stringify
		let string_repr = codec.stringify(&test_key);
		assert_eq!(string_repr, format!("hexBytes:{}", hex::encode(&test_key)));

		// Test key_type
		assert_eq!(codec.key_type(), "bytes");

		// Test max size limit
		let too_long_key = vec![0u8; (MAX_BYTES_KEY_SIZE as usize) + 1];
		let mut large_buffer = vec![0u8; too_long_key.len()];
		assert!(codec.encode(&mut large_buffer, &too_long_key).is_err());
	}

	#[test]
	fn test_bytes_value_codec() {
		let codec = BytesValueCodec;
		let test_value = vec![1, 2, 3, 4, 5];

		// Test encoding
		let encoded = codec.encode(&test_value).unwrap();
		assert_eq!(
			encoded, test_value,
			"encoded bytes should match original value"
		);

		// Test decoding
		let decoded = codec.decode(&encoded).unwrap();
		assert_eq!(
			decoded, test_value,
			"decoded value should match original value"
		);

		// Test stringify
		let string_repr = codec.stringify(&test_value);
		assert_eq!(
			string_repr,
			format!("hexBytes:{}", hex::encode(&test_value))
		);

		// Test value_type
		assert_eq!(codec.value_type(), "bytes");

		// Test with empty value
		let empty_value = vec![];
		let encoded_empty = codec.encode(&empty_value).unwrap();
		assert_eq!(encoded_empty, empty_value);
		let decoded_empty = codec.decode(&encoded_empty).unwrap();
		assert_eq!(decoded_empty, empty_value);

		// Test with large value
		let large_value = vec![0u8; 1000];
		let encoded_large = codec.encode(&large_value).unwrap();
		assert_eq!(encoded_large, large_value);
		let decoded_large = codec.decode(&encoded_large).unwrap();
		assert_eq!(decoded_large, large_value);
	}
}
