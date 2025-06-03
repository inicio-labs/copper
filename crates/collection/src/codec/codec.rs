use crate::error::CollectionError;

/// KeyCodec defines traits for types that can encode and decode collection keys.
pub trait KeyCodec<T> {
	/// Encode writes the key bytes into the buffer. Returns the number of
	/// bytes written. The implementer must expect the buffer to be at least
	/// of length equal to Size(T) for all encodings.
	///
	/// It must also return the number of written bytes which must be
	/// equal to Size(T) for all encodings not involving varints.
	/// In case of encodings involving varints then the returned
	/// number of written bytes is allowed to be smaller than Size(T).
	fn encode(&self, buffer: &mut Vec<u8>, key: &T) -> Result<usize, CollectionError>;

	/// Decode reads from the provided bytes buffer to decode
	/// the key T. Returns the number of bytes read and the decoded value,
	/// or an error in case of decoding failure.
	fn decode(&self, buffer: &Vec<u8>) -> Result<(usize, T), CollectionError>;

	/// Size returns the buffer size needed to encode key T in binary format.
	/// The returned value must match what is computed by encode for all
	/// encodings except the ones involving varints. Varints are expected
	/// to return the maximum varint bytes buffer length, at the risk of
	/// over-estimating in order to pick the most performant path.
	fn size(&self, key: &T) -> usize;

	/// Stringify returns a string representation of T.
	fn stringify(&self, key: &T) -> String;

	/// KeyType returns a string identifier for the type of the key.
	fn key_type(&self) -> String;
}

/// ValueCodec defines traits for types that can encode and decode collection values.
pub trait ValueCodec<T> {
	/// Encode encodes the value T into binary format.
	fn encode(&self, value: &T) -> Result<Vec<u8>, CollectionError>;

	/// Decode returns the type T given its binary representation.
	fn decode(&self, bytes: &Vec<u8>) -> Result<T, CollectionError>;

	/// Stringify returns a string representation of T.
	fn stringify(&self, value: &T) -> String;

	/// ValueType returns the identifier for the type.
	fn value_type(&self) -> String;
}
