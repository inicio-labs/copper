use crate::context::Context;

#[derive(Clone)]
pub struct MockContext {}

impl Context for MockContext {}

impl MockContext {
	pub fn new() -> Self {
		Self {}
	}
}
