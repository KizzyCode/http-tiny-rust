//! A data implementation that ensures that it's bytes correspond to a particular encoding like
//! UTF-8, ASCII etc.
//!
//! This mod is used to constrain the HTTP header parts to their context-specific encoding.

pub mod encodings;

use crate::{
	HttpError,
	data::encodings::{ Encoding, Utf8Subset, Integer }
};
use std::{
	str, u128, convert::TryFrom, marker::PhantomData, num::ParseIntError, ops::Deref,
	hash::{ Hash, Hasher }, fmt::{ self, Display, Formatter }
};


/// A helper macro to convert a static `$str` into a `Data`-target using the `TryInto`-trait
///
/// _Panics if `try_into` fails_
#[macro_export] macro_rules! data {
	($str:expr) => ({
		let str: &'static str = $str;
		::std::convert::TryInto::try_into(str).unwrap()
	});
}


/// Some data that conforms to a specific encoding
#[derive(Copy, Clone, Debug)]
pub struct Data<'a, E: Encoding> {
	underlying: &'a[u8],
	_mode: PhantomData<E>
}
impl<'a, E: Encoding> Data<'a, E> {
	/// Validates that `bytes` conform to the encoding `M`
	pub fn validate(bytes: &[u8]) -> bool {
		E::is_valid(bytes)
	}
	/// Returns a reference to the underlying slice
	pub fn as_slice(&self) -> &'a[u8] {
		self.underlying
	}
}
impl<'a, E: Encoding> Deref for Data<'a, E> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		self.underlying
	}
}
impl<'a, E: Encoding> AsRef<[u8]> for Data<'a, E> {
	fn as_ref(&self) -> &[u8] {
		self.underlying
	}
}
impl<'a, E: Encoding + Utf8Subset> AsRef<str> for Data<'a, E> {
	fn as_ref(&self) -> &str {
		str::from_utf8(self.underlying).unwrap()
	}
}
impl<'a, E: Encoding + Utf8Subset> Display for Data<'a, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}
impl<'a, 'b, E: Encoding> PartialEq<Data<'b, E>> for Data<'a, E> {
	fn eq(&self, other: &Data<'b, E>) -> bool {
		E::is_eq(self, other)
	}
}
impl<'a, E: Encoding> PartialEq<&str> for Data<'a, E> {
	fn eq(&self, other: &&str) -> bool {
		E::is_eq(self, other.as_bytes())
	}
}
impl<'a, E: Encoding> PartialEq<Data<'a, E>> for &str {
	fn eq(&self, other: &Data<'a, E>) -> bool {
		E::is_eq(self.as_bytes(), other)
	}
}
impl<'a, E: Encoding> Eq for Data<'a, E> {}
impl<'a, E: Encoding> Hash for Data<'a, E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		E::hash(self.underlying, state)
	}
}
impl<'a, 'b: 'a, E: Encoding> TryFrom<&'b str> for Data<'a, E> {
	type Error = HttpError;
	fn try_from(source: &'b str) -> Result<Self, Self::Error> {
		Self::try_from(source.as_bytes())
	}
}
impl<'a, 'b: 'a, E: Encoding> TryFrom<&'b[u8]> for Data<'a, E> {
	type Error = HttpError;
	fn try_from(bytes: &'b[u8]) -> Result<Self, Self::Error> {
		match Self::validate(bytes) {
			true => Ok(Self{ underlying: bytes, _mode: PhantomData }),
			false => Err(HttpError::InvalidEncoding)
		}
	}
}
impl<'a> TryFrom<Data<'a, Integer>> for u128 {
	type Error = ParseIntError;
	fn try_from(data: Data<'a, Integer>) -> Result<Self, Self::Error> {
		Self::from_str_radix(data.as_ref(), 10)
	}
}
impl<'a> TryFrom<Data<'a, Integer>> for u16 {
	type Error = ParseIntError;
	fn try_from(data: Data<'a, Integer>) -> Result<Self, Self::Error> {
		Self::from_str_radix(data.as_ref(), 10)
	}
}
