use core::cmp::Ordering;
use std::collections::BinaryHeap;

use malachitebft_app_channel::app::streaming::StreamMessage;

#[derive(Debug)]
pub struct MinHeap<T>(BinaryHeap<MinSeq<T>>);

#[derive(Debug)]
struct MinSeq<T>(StreamMessage<T>);

impl<T> MinHeap<T> {
	pub fn push(&mut self, msg: StreamMessage<T>) {
		self.0.push(MinSeq(msg));
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn drain(&mut self) -> Vec<T> {
		let mut drained = Vec::with_capacity(self.0.len());

		while let Some(MinSeq(msg)) = self.0.pop() {
			if let Some(data) = msg.content.into_data() {
				drained.push(data);
			}
		}

		drained
	}
}

impl<T> Default for MinHeap<T> {
	fn default() -> Self {
		Self(BinaryHeap::new())
	}
}

impl<T> PartialEq for MinSeq<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.sequence == other.0.sequence
	}
}

impl<T> Eq for MinSeq<T> {}

impl<T> PartialOrd for MinSeq<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<T> Ord for MinSeq<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		other.0.sequence.cmp(&self.0.sequence)
	}
}
