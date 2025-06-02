mod bytes;
mod codec;
mod error;

pub use bytes::{BytesCodec, MAX_BYTES_KEY_SIZE};
pub use codec::KeyCodec;
pub use error::CodecError;
