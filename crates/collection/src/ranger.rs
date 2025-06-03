pub struct Ranger<K> {
	pub start: Option<K>,
	pub end: Option<K>,
	pub direction: Direction,
}

pub enum Direction {
	Asc,
	Desc,
}
