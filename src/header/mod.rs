//! A HTTP header implementation that can parse, create and serialize arbitrary HTTP/1.0 and
//! HTTP/1.1 request and response headers.

mod builders;
mod header;

pub use self::{
	builders::{ RequestBuilder, ResponseBuilder },
	header::{ RequestHeader, ResponseHeader, read, parse_request, parse_response }
};