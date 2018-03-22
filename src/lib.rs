#[macro_use] extern crate etrace;
extern crate io;

pub mod header;
pub mod chunked_body;

use etrace::Error;
pub use header::{ Header, RequestHeaderView, ResponseHeaderView };



#[derive(Debug, Clone)]
pub enum HttpError {
	IoError,
	ProtocolError,
	ApiMisuse
}
impl From<std::string::FromUtf8Error> for HttpError {
	fn from(_: std::string::FromUtf8Error) -> Self {
		HttpError::ProtocolError
	}
}
impl From<std::num::ParseIntError> for HttpError {
	fn from(_: std::num::ParseIntError) -> Self {
		HttpError::ProtocolError
	}
}