/// An extension for iterators
pub trait IterExt: Iterator {
	/// Collects the iterator and ensures that we've collected `n` items at min
	fn collect_min(self, n: usize) -> Option<Vec<Self::Item>>;
	/// Collects the iterator and ensures that we've collected `n` items at max
	fn collect_max(self, n: usize) -> Option<Vec<Self::Item>>;
	/// Collects the iterator and ensures that we've collected *exactly* `n` items
	fn collect_exact(self, n: usize) -> Option<Vec<Self::Item>>;
}
impl<T: Iterator> IterExt for T {
	fn collect_min(self, n: usize) -> Option<Vec<Self::Item>> {
		let items: Vec<Self::Item> = self.collect();
		match items.len() >= n {
			true => Some(items),
			false => None
		}
	}
	fn collect_max(mut self, n: usize) -> Option<Vec<Self::Item>> {
		// Collect up to `n` items
		let mut items = Vec::new();
		while items.len() < n {
			match self.next() {
				Some(item) => items.push(item),
				None => break
			}
		}
		// Ensure that the iterator is now emtpy
		match self.next() {
			None => Some(items),
			Some(_) => None
		}
	}
	fn collect_exact(self, n: usize) -> Option<Vec<Self::Item>> {
		let items: Vec<Self::Item> = self.collect_max(n)?;
		match items.len() == n {
			true => Some(items),
			false => None
		}
	}
}