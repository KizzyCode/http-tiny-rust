//! A data implementation that ensures that it's bytes correspond to a particular encoding like
//! UTF-8, ASCII etc.
//!
//! This mod is used to constrain the HTTP header parts to their context-specific encoding.

pub mod encodings;

use crate::{
	HttpError,
	data::encodings::{ Encoding, SubsetOf, Utf8, Integer }
};
use std::{
	str, u128, convert::TryFrom, marker::PhantomData, num::ParseIntError, ops::Deref,
	hash::{ Hash, Hasher }, fmt::{ self, Display, Formatter }
};


/// A helper macro to convert a static `$str` into a `Data`-target using the `TryInto`-trait
///
/// _Panics if `try_into` fails_
#[macro_export] macro_rules! data {
	($str:expr) => (::std::convert::TryInto::try_into($str).unwrap());
}


/// Some data that conforms to a specific encoding
#[derive(Clone, Debug)]
pub struct Data<E: Encoding> {
	bytes: Vec<u8>,
	_encoding: PhantomData<E>
}
impl<E: Encoding> Data<E> {
	/// Tries to create `self` from data with a superset encoding
	pub fn try_from_superset<S: Encoding>(superset: Data<S>) -> Result<Self, HttpError>
		where E: SubsetOf<S>
	{
		Self::try_from(superset.bytes)
	}
	/// Creates `self` from data with a subset encoding
	pub fn from_subset<S: Encoding>(subset: Data<S>) -> Self where S: SubsetOf<E> {
		Self::try_from(subset.bytes).unwrap()
	}
}
impl<E: Encoding> Data<E> {
	/// Validates that `bytes` conform to the encoding `M`
	pub fn validate(bytes: &[u8]) -> bool {
		E::is_valid(bytes)
	}
}
impl<E: Encoding> Deref for Data<E> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.bytes
	}
}
impl<E: Encoding> AsRef<[u8]> for Data<E> {
	fn as_ref(&self) -> &[u8] {
		self
	}
}
impl<E: Encoding + SubsetOf<Utf8>> AsRef<str> for Data<E> {
	fn as_ref(&self) -> &str {
		str::from_utf8(self).unwrap()
	}
}
impl<E: Encoding + SubsetOf<Utf8>> Display for Data<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}
impl<E: Encoding> PartialEq for Data<E> {
	fn eq(&self, other: &Data<E>) -> bool {
		E::is_eq(self, other)
	}
}
impl<E: Encoding> Eq for Data<E> {}
impl<E: Encoding> Hash for Data<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		E::hash(self, state)
	}
}
impl<E: Encoding> PartialEq<str> for Data<E> {
	fn eq(&self, other: &str) -> bool {
		E::is_eq(self, other.as_bytes())
	}
}
impl<E: Encoding> PartialEq<Data<E>> for str {
	fn eq(&self, other: &Data<E>) -> bool {
		E::is_eq(self.as_bytes(), other)
	}
}


impl<E: Encoding> TryFrom<&str> for Data<E> {
	type Error = HttpError;
	fn try_from(source: &str) -> Result<Self, Self::Error> {
		Self::try_from(source.to_string())
	}
}
impl<E: Encoding> TryFrom<&[u8]> for Data<E> {
	type Error = HttpError;
	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		Self::try_from(bytes.to_vec())
	}
}
impl<E: Encoding> TryFrom<String> for Data<E> {
	type Error = HttpError;
	fn try_from(source: String) -> Result<Self, Self::Error> {
		Self::try_from(source.into_bytes())
	}
}
impl<E: Encoding> TryFrom<Vec<u8>> for Data<E> {
	type Error = HttpError;
	fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
		match Self::validate(&bytes) {
			true => Ok(Self{ bytes, _encoding: PhantomData }),
			false => Err(HttpError::InvalidEncoding)
		}
	}
}


impl<E: Encoding> From<Data<E>> for Vec<u8> {
	fn from(data: Data<E>) -> Self {
		data.bytes
	}
}
impl TryFrom<Data<Integer>> for u128 {
	type Error = ParseIntError;
	fn try_from(data: Data<Integer>) -> Result<Self, Self::Error> {
		Self::from_str_radix(data.as_ref(), 10)
	}
}
impl<'a> TryFrom<Data<Integer>> for u16 {
	type Error = ParseIntError;
	fn try_from(data: Data<Integer>) -> Result<Self, Self::Error> {
		Self::from_str_radix(data.as_ref(), 10)
	}
}
