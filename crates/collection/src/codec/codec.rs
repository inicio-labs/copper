use super::error::CodecError;

/// KeyCodec defines a generic interface for types that can encode and decode collection keys.
pub trait KeyCodec<T> {
	/// Encode writes the key bytes into the buffer. Returns the number of
	/// bytes written. The implementer must expect the buffer to be at least
	/// of length equal to Size(T) for all encodings.
	///
	/// It must also return the number of written bytes which must be
	/// equal to Size(T) for all encodings not involving varints.
	/// In case of encodings involving varints then the returned
	/// number of written bytes is allowed to be smaller than Size(T).
	fn encode(&self, buffer: &mut [u8], key: &T) -> Result<usize, CodecError>;

	/// Decode reads from the provided bytes buffer to decode
	/// the key T. Returns the number of bytes read and the decoded value,
	/// or an error in case of decoding failure.
	fn decode(&self, buffer: &[u8]) -> Result<(usize, T), CodecError>;

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
