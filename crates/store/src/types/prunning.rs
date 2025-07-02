/// Pruning options for store management
#[derive(Debug, Clone)]
pub struct PruningOptions {
	pub keep_recent: u64,
	pub interval: u64,
	pub strategy: PruningStrategy,
}

#[derive(Debug, Clone)]
pub enum PruningStrategy {
	Default,
	Everything,
	Nothing,
	Custom,
}
