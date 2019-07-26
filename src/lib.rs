mod helpers;
mod query_string;
mod header;
pub mod data;

use std::{
	error::Error,
	fmt::{ self, Display, Formatter }
};
pub use crate::{
	query_string::QueryString,
	header::{
		builders::{ RequestBuilder, ResponseBuilder },
		header::{ Header, RequestHeader, ResponseHeader }
	}
};


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