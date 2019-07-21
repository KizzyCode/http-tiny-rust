mod helpers;
mod owned_ref;
mod query_string;
pub mod data;
pub mod header;

use std::{
	error::Error,
	fmt::{ self, Display, Formatter }
};
pub use crate::{ owned_ref::OwnedRef, query_string::QueryString };


/// A `http_header` related error
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HttpError {
	InvalidEncoding,
	TruncatedData,
	ProtocolViolation,
	ApiMisuse
}
impl Display for HttpError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl Error for HttpError {}