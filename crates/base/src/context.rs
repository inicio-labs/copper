use crate::block::BlockHeight;

#[derive(Debug)]
pub struct MutContext<'t, 's, S> {
	height: BlockHeight,
	tx_bz: &'t [u8],
	store: &'s mut S,
}

impl<'t, 's, S> MutContext<'t, 's, S> {
	pub fn new(height: BlockHeight, tx_bz: &'t [u8], store: &'s mut S) -> Self {
		Self { height, tx_bz, store }
	}

	pub fn height(&self) -> BlockHeight {
		self.height
	}

	pub fn tx_bz(&self) -> &'t [u8] {
		self.tx_bz
	}

	pub fn store(&mut self) -> &mut S {
		self.store
	}
}
