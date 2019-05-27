use std::fmt::{Debug, Formatter};
use std::fmt;


/// An iterator over splitted sub-slices
pub struct Splitter<'a, T: PartialEq> {
	data: &'a[T],
	pat: &'a[T],
	remaining: usize
}
impl<'a, T: PartialEq> Iterator for Splitter<'a, T> {
	type Item = &'a[T];
	fn next(&mut self) -> Option<&'a[T]> {
		// Takes the next slice and updates the state
		macro_rules! take {
			(next: $len:expr) => ({
				let (slice, remaining) = self.data.split_at($len);
				self.data = remaining.split_at(self.pat.len()).1;
				self.remaining -= 1;
				slice
			});
			(last) => ({
				self.remaining = 0;
				self.data
			});
		}
		
		// Get the next slice
		match self.remaining {
			0 => None,
			1 => Some(take!(last)),
			_ => {
				// Find the next pattern and get the slice
				let next_pat = (0 .. self.data.len())
					.find(|i| self.data[*i..]
					.starts_with(self.pat));
				Some(match next_pat {
					Some(next_pat) => take!(next: next_pat),
					None => take!(last)
				})
			}
		}
	}
}
impl<'a, T: PartialEq + Debug> Debug for Splitter<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Splitter")
			.field("data", &self.data)
			.field("pat", &self.pat)
			.field("remaining", &self.remaining)
			.finish()
	}
}


/// An extension for slices
pub trait SliceExt<'a, T: PartialEq> {
	/// Splits the slice by `pat`
	fn split_pat(&'a self, pat: &'a AsRef<[T]>) -> Splitter<'a, T>;
	/// Splits the slice `n` times by `pat`
	fn splitn_pat(&'a self, n: usize, pat: &'a AsRef<[T]>) -> Splitter<'a, T>;
}
impl<'a, T: PartialEq + Clone> SliceExt<'a, T> for [T] {
	fn split_pat(&'a self, pat: &'a AsRef<[T]>) -> Splitter<'a, T> {
		self.splitn_pat(usize::max_value(), pat)
	}
	fn splitn_pat(&'a self, n: usize, pat: &'a AsRef<[T]>) -> Splitter<'a, T> {
		Splitter{ data: self, pat: pat.as_ref(), remaining: n }
	}
}


pub trait ByteSliceExt<'a> {
	/// Returns a slice without the leading whitespace bytes
	///
	/// "Whitespace" is defined as an ASCII whitespace character:
	/// U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM FEED, or U+000D CARRIAGE
	/// RETURN.
	fn trim_start(&'a self) -> &'a Self;
	/// Returns a slice without the trailing whitespace bytes
	///
	/// "Whitespace" is defined as an ASCII whitespace character:
	/// U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM FEED, or U+000D CARRIAGE
	/// RETURN.
	fn trim_end(&'a self) -> &'a Self;
	/// Returns a slice without the leading and trailing whitespace bytes
	///
	/// "Whitespace" is defined as an ASCII whitespace character:
	/// U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM FEED, or U+000D CARRIAGE
	/// RETURN.
	fn trim(&'a self) -> &'a Self {
		self.trim_start().trim_end()
	}
}
impl<'a> ByteSliceExt<'a> for [u8] {
	fn trim_start(&'a self) -> &'a Self {
		let len = self.iter()
			.take_while(|b| b.is_ascii_whitespace()).count();
		self.split_at(len).1
	}
	fn trim_end(&'a self) -> &'a Self {
		let len = self.iter().rev()
			.take_while(|b| b.is_ascii_whitespace()).count();
		self.split_at(self.len() - len).0
	}
}