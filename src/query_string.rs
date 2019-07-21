use crate::{
	HttpError,
	helpers::{ iter_ext::IterExt, slice_ext::SliceExt },
	data::{
		Data,
		encodings::{ Encoding, Uri, UriQuery}
	}
};
use std::{
	collections::HashMap, io::Write,
	convert::{ TryFrom, TryInto }
};
use crate::helpers::slice_ext::ByteSliceExt;


/// A [query string](https://tools.ietf.org/html/rfc3986#section-3.4)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryString(HashMap<Data<UriQuery>, Data<UriQuery>>);
impl QueryString {
	/// Create a new `QueryUri` instance
	pub fn new() -> Self {
		Self(HashMap::new())
	}
	
	/// Gets a reference to the field for `key`
	pub fn field(&self, key: &Data<UriQuery>) -> Option<&Data<UriQuery>> {
		self.0.get(&key)
	}
	/// Gets a mutable reference to field for `key`
	pub fn field_mut(&mut self, key: &Data<UriQuery>)
		-> Option<&mut Data<UriQuery>>
	{
		self.0.get_mut(&key)
	}
	/// Inserts a new field with `key` and `value`
	pub fn insert(&mut self, key: Data<UriQuery>, value: Data<UriQuery>) {
		self.0.insert(key, value);
	}
	
	/// A reference to the fields
	pub fn fields(&self) -> &HashMap<Data<UriQuery>, Data<UriQuery>> {
		&self.0
	}
	/// A mutable reference to the fields
	pub fn fields_mut(&mut self) -> &mut HashMap<Data<UriQuery>, Data<UriQuery>> {
		&mut self.0
	}
	
	/// Creates a query string from the stored key-value pairs
	pub fn to_string(&self) -> String {
		// Create the query "string"
		let mut query = vec![b'?'];
		self.0.iter().for_each(|(k, v)| match v.len() {
			0 => write!(&mut query, "{}&", k).unwrap(),
			_ => write!(&mut query, "{}={}&", k, v).unwrap()
		});
		
		// Remove last ampersand
		let trimmed_len = query.trim_end_matches(|b| *b == b'&').len();
		query.truncate(trimmed_len);
		String::from_utf8(query).unwrap()
	}
}
impl TryFrom<Data<Uri>> for QueryString {
	type Error = HttpError;
	fn try_from(uri: Data<Uri>) -> Result<Self, Self::Error> {
		// Cut-off the part before the query string and remove an optional fragment appendix
		let query_part = match uri.splitn_pat(2, b"?").collect_min(2) {
			Some(query_part) => query_part[1],
			None => return Ok(Self::new())
		};
		let query_part = query_part.splitn_pat(2, b"#").next().unwrap();
		
		// Validate the encoding before the next parsing step and trim the trailing ampersands
		if !UriQuery::is_valid(query_part) {
			Err(HttpError::InvalidEncoding)?
		}
		let query_part = query_part.trim_end_matches(|b| *b == b'&');
		if query_part.is_empty() {
			return Ok(Self::new())
		}
		
		// Split the query into key-value parts
		let mut query = HashMap::new();
		for kv in query_part.split_pat(b"&") {
			let kv: Vec<&[u8]> = kv.splitn_pat(2, b"=").collect();
			match kv.len() {
				1 => query.insert(kv[0].try_into()?, b"".as_ref().try_into()?),
				2 => query.insert(kv[0].try_into()?, kv[1].try_into()?),
				_ => unreachable!()
			};
		}
		Ok(Self(query))
	}
}