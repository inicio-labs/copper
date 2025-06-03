pub trait Collection {
	fn get_name(&self) -> String;

	fn get_prefix(&self) -> Vec<u8>;
}
