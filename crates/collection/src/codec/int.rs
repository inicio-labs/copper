use crate::{
	codec::{KeyCodec, ValueCodec},
	CollectionError,
};
use byteorder::{BigEndian, ByteOrder};

#[derive(Debug, Clone, Copy, Default)]

pub struct I64KeyCodec;

impl KeyCodec<i64> for I64KeyCodec {
	fn encode(&self, buffer: &mut Vec<u8>, key: &i64) -> Result<usize, CollectionError> {
		let mut buf = [0u8; 8];

		let key_u64 = *key as u64;

		BigEndian::write_u64(&mut buf, key_u64);

		buf[0] = buf[0] ^ 0x80;

		buffer.extend(buf);

		Ok(8)
	}

	fn decode(&self, buffer: &Vec<u8>) -> Result<(usize, i64), CollectionError> {
		if buffer.len() != 8 {
			return Err(CollectionError::DecodeError(
				"invalid int bytes length".to_string(),
			));
		}

		let u = (buffer[7] as u64)
			| ((buffer[6] as u64) << 8)
			| ((buffer[5] as u64) << 16)
			| ((buffer[4] as u64) << 24)
			| ((buffer[3] as u64) << 32)
			| ((buffer[2] as u64) << 40)
			| ((buffer[1] as u64) << 48)
			| ((buffer[0] ^ 0x80) as u64) << 56;

		Ok((8, u as i64))
	}

	fn size(&self, _key: &i64) -> usize {
		8
	}

	// TODO: this is not correct, we need to use a proper stringifier
	fn stringify(&self, key: &i64) -> String {
		key.to_string()
	}

	fn key_type(&self) -> String {
		"int64".to_string()
	}
}

#[derive(Debug, Clone, Copy, Default)]

pub struct I64ValueCodec;

impl ValueCodec<i64> for I64ValueCodec {
	fn encode(&self, value: &i64) -> Result<Vec<u8>, CollectionError> {
		let mut buffer = [0u8; 8];

		let value_u64 = *value as u64;

		BigEndian::write_u64(&mut buffer, value_u64);

		buffer[0] = buffer[0] ^ 0x80;

		Ok(buffer.to_vec())
	}

	fn decode(&self, bytes: &Vec<u8>) -> Result<i64, CollectionError> {
		if bytes.len() != 8 {
			return Err(CollectionError::DecodeError(
				"invalid int bytes length".to_string(),
			));
		}

		let u = (bytes[7] as u64)
			| ((bytes[6] as u64) << 8)
			| ((bytes[5] as u64) << 16)
			| ((bytes[4] as u64) << 24)
			| ((bytes[3] as u64) << 32)
			| ((bytes[2] as u64) << 40)
			| ((bytes[1] as u64) << 48)
			| ((bytes[0] ^ 0x80) as u64) << 56;

		Ok(u as i64)
	}

	// TODO: this is not correct, we need to use a proper stringifier
	fn stringify(&self, value: &i64) -> String {
		value.to_string()
	}

	fn value_type(&self) -> String {
		"int64".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_i64_key_codec() {
		let codec = I64KeyCodec::default();

		// Test various numbers including edge cases
		let test_cases = vec![0i64, 1i64, i64::MAX, i64::MIN, 0xFFFFFFFFFFFFFFFi64];

		for value in test_cases {
			let mut buffer = Vec::new();

			// Test encode
			let size = codec.encode(&mut buffer, &value).unwrap();
			assert_eq!(size, 8, "Encoded size should be 8 bytes");

			// Test decode
			let (read_size, decoded) = codec.decode(&buffer).unwrap();
			assert_eq!(read_size, 8, "Decoded size should be 8 bytes");
			assert_eq!(decoded, value, "Decoded value should match original");

			// Test size
			assert_eq!(codec.size(&value), 8, "Size should be 8 bytes");
		}
	}

	#[test]
	fn test_i64_value_codec() {
		let codec = I64ValueCodec::default();

		// Test various numbers including edge cases
		let test_cases = vec![0i64, 1i64, i64::MAX, i64::MIN, 123456789i64, i64::MAX];

		for value in test_cases {
			// Test encode
			let encoded = codec.encode(&value).unwrap();
			assert_eq!(encoded.len(), 8, "Encoded length should be 8 bytes");

			// Test decode
			let decoded = codec.decode(&encoded).unwrap();
			assert_eq!(decoded, value, "Decoded value should match original");
		}
	}

	#[test]
	fn test_ordering() {
		let codec = I64KeyCodec::default();
		let numbers = vec![0i64, 1, 5, 10, 100, i64::MAX];
		let mut encoded_values = Vec::new();

		// Encode all numbers
		for &num in &numbers {
			let mut buffer = Vec::new();
			codec.encode(&mut buffer, &num).unwrap();
			encoded_values.push(buffer);
		}

		// Test that encoded values maintain the same ordering
		for i in 0..encoded_values.len() - 1 {
			println!("{:?} < {:?}", encoded_values[i], encoded_values[i + 1]);

			assert!(
				encoded_values[i] < encoded_values[i + 1],
				"Encoded values should maintain ordering"
			);
		}
	}

	#[test]
	fn test_invalid_decode() {
		let key_codec = I64KeyCodec::default();
		let value_codec = I64ValueCodec::default();

		// Test with invalid length
		let invalid_buffer = vec![1, 2, 3]; // Less than 8 bytes

		assert!(key_codec.decode(&invalid_buffer).is_err());
		assert!(value_codec.decode(&invalid_buffer).is_err());
	}

	#[test]
	fn test_stringify_and_type() {
		let key_codec = I64KeyCodec::default();
		let value_codec = I64ValueCodec::default();

		assert_eq!(key_codec.stringify(&123i64), "123");
		assert_eq!(value_codec.stringify(&456i64), "456");

		assert_eq!(key_codec.key_type(), "int64");
		assert_eq!(value_codec.value_type(), "int64");
	}
}
