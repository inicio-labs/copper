use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Header {
	pub height: i64,
	pub time: std::time::SystemTime,
}

// TODO
impl Default for Header {
	fn default() -> Self {
		Self { height: 0, time: SystemTime::UNIX_EPOCH }
	}
}
