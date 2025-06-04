use crate::context::Context;

pub struct ContextImpl {}

impl Context for ContextImpl {}

impl ContextImpl {
	pub fn new() -> Self {
		Self {}
	}
}
