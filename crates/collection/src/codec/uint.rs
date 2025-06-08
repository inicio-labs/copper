use crate::{
	codec::{KeyCodec, ValueCodec},
	CollectionError,
};
use byteorder::{BigEndian, ByteOrder};

#[derive(Debug, Clone, Copy, Default)]

pub struct U64KeyCodec;

impl KeyCodec<u64> for U64KeyCodec {
	fn encode(&self, buffer: &mut Vec<u8>, key: &u64) -> Result<usize, CollectionError> {
		let mut buf = [0u8; 8];
		let key_u64 = *key as u64;

		BigEndian::write_u64(&mut buf, key_u64);
		buffer.extend(buf);

		Ok(8)
	}

	fn decode(&self, buffer: &Vec<u8>) -> Result<(usize, u64), CollectionError> {
		if buffer.len() != 8 {
			return Err(CollectionError::DecodeError(
				"invalid int bytes length".to_string(),
			));
		}

		let u = BigEndian::read_u64(buffer.as_slice());

		Ok((8, u))
	}

	fn size(&self, _key: &u64) -> usize {
		8
	}

	// TODO: this is not correct, we need to use a proper stringifier
	fn stringify(&self, key: &u64) -> String {
		key.to_string()
	}

	fn key_type(&self) -> String {
		"uint64".to_string()
	}
}

#[derive(Debug, Clone, Copy, Default)]

pub struct U64ValueCodec;

impl ValueCodec<u64> for U64ValueCodec {
	fn encode(&self, value: &u64) -> Result<Vec<u8>, CollectionError> {
		let mut buffer = [0u8; 8];

		let value_u64 = *value as u64;

		BigEndian::write_u64(&mut buffer, value_u64);

		Ok(buffer.to_vec())
	}

	fn decode(&self, bytes: &Vec<u8>) -> Result<u64, CollectionError> {
		if bytes.len() != 8 {
			return Err(CollectionError::DecodeError(
				"invalid int bytes length".to_string(),
			));
		}

		let u = BigEndian::read_u64(bytes.as_slice());

		Ok(u)
	}

	// TODO: this is not correct, we need to use a proper stringifier
	fn stringify(&self, value: &u64) -> String {
		value.to_string()
	}

	fn value_type(&self) -> String {
		"uint64".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_u64_key_codec_encode() {
		let codec = U64KeyCodec::default();
		let mut buffer = Vec::new();

		// Test encoding different values
		let test_cases = vec![0u64, 1u64, u64::MAX, 42u64, 1234567890u64];

		for value in test_cases {
			buffer.clear();
			let size = codec.encode(&mut buffer, &value).unwrap();
			assert_eq!(size, 8);
			assert_eq!(buffer.len(), 8);

			// Verify the encoded bytes using BigEndian::read_u64
			let decoded = BigEndian::read_u64(&buffer);
			assert_eq!(decoded, value);
		}
	}

	#[test]
	fn test_u64_key_codec_decode() {
		let codec = U64KeyCodec::default();
		let test_cases = vec![0u64, 1u64, u64::MAX, 42u64, 1234567890u64];

		for expected in test_cases {
			let mut buffer = vec![0u8; 8];
			BigEndian::write_u64(&mut buffer, expected);
			let (size, value) = codec.decode(&buffer).unwrap();
			assert_eq!(size, 8);
			assert_eq!(value, expected);
		}
	}

	#[test]
	fn test_u64_key_codec_decode_invalid_length() {
		let codec = U64KeyCodec::default();
		let invalid_buffer = vec![1, 2, 3]; // Less than 8 bytes

		match codec.decode(&invalid_buffer) {
			Err(CollectionError::DecodeError(msg)) => {
				assert_eq!(msg, "invalid int bytes length");
			},
			_ => panic!("Expected DecodeError for invalid buffer length"),
		}
	}

	#[test]
	fn test_u64_value_codec_encode_decode() {
		let codec = U64ValueCodec::default();
		let test_cases = vec![0u64, 1u64, u64::MAX, 42u64, 1234567890u64];

		for value in test_cases {
			let encoded = codec.encode(&value).unwrap();
			assert_eq!(encoded.len(), 8);

			let decoded = codec.decode(&encoded).unwrap();
			assert_eq!(decoded, value);
		}
	}

	#[test]
	fn test_u64_value_codec_decode_invalid_length() {
		let codec = U64ValueCodec::default();
		let invalid_buffer = vec![1, 2, 3]; // Less than 8 bytes

		match codec.decode(&invalid_buffer) {
			Err(CollectionError::DecodeError(msg)) => {
				assert_eq!(msg, "invalid int bytes length");
			},
			_ => panic!("Expected DecodeError for invalid buffer length"),
		}
	}

	#[test]
	fn test_stringify_and_types() {
		let key_codec = U64KeyCodec::default();
		let value_codec = U64ValueCodec::default();

		// Test stringify
		assert_eq!(key_codec.stringify(&42), "42");
		assert_eq!(value_codec.stringify(&42), "42");

		// Test type names
		assert_eq!(key_codec.key_type(), "uint64");
		assert_eq!(value_codec.value_type(), "uint64");
	}

	#[test]
	fn test_size() {
		let codec = U64KeyCodec::default();
		assert_eq!(codec.size(&42), 8);
		assert_eq!(codec.size(&0), 8);
		assert_eq!(codec.size(&u64::MAX), 8);
	}
}
