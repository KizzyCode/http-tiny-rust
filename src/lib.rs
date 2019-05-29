mod helpers;
mod query_string;
pub mod data;
pub mod header;

use std::{
	error::Error,
	fmt::{ self, Display, Formatter }
};
pub use crate::query_string::QueryString;


/// A `http` related error
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HttpError {
	InvalidEncoding,
	TruncatedData,
	ProtocolViolation,
	ApiMisuse,
	Debug
}
impl Display for HttpError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl Error for HttpError {}