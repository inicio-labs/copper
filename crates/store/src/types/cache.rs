pub trait CacheWrap {
	fn write(&mut self);
	fn cache_wrap(&self) -> Box<dyn CacheWrap>;
}
