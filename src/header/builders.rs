use crate::{
	HttpError,
	header::{ RequestHeader, ResponseHeader, header::Header },
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey, Uri, Integer }
	}
};
use std::collections::HashMap;


/// A HTTP-response header builder
pub struct RequestBuilder<'a> {
	method: Option<Data<'a, Ascii>>,
	uri: Option<Data<'a, Uri>>,
	version: Option<Data<'a, Ascii>>,
	header_fields: HashMap<Data<'a, HeaderFieldKey>, Data<'a, Ascii>>
}
impl<'a> RequestBuilder<'a> {
	/// Creates a new request builder
	pub fn new() -> Self {
		Self{ method: None, uri: None, version: None, header_fields: HashMap::new() }
	}
	
	/// Sets the request method
	pub fn method(mut self, method: Data<'a, Ascii>) -> Self {
		self.method = Some(method);
		self
	}
	/// Sets the request URI
	pub fn uri(mut self, uri: Data<'a, Uri>) -> Self {
		self.uri = Some(uri);
		self
	}
	/// Sets the HTTP version
	pub fn version(mut self, version: Data<'a, Ascii>) -> Self {
		self.version = Some(version);
		self
	}
	
	/// Inserts a header field with `key`-`value`
	pub fn field(mut self, key: Data<'a, HeaderFieldKey>, value: Data<'a, Ascii>) -> Self {
		self.header_fields.insert(key, value);
		self
	}
	
	/// Builds the request header
	pub fn build(self) -> Result<RequestHeader<'a>, HttpError> {
		// Unwrap status fields
		let method = self.method.ok_or(HttpError::ApiMisuse)?;
		let uri = self.uri.ok_or(HttpError::ApiMisuse)?;
		let version = self.version.ok_or(HttpError::ApiMisuse)?;
		
		Ok(RequestHeader(Header {
			header_line: (method.as_slice(), uri.as_slice(), version.as_slice()),
			header_fields: self.header_fields
		}))
	}
}


/// A HTTP-response header builder
pub struct ResponseBuilder<'a> {
	version: Option<Data<'a, Ascii>>,
	status: Option<Data<'a, Integer>>,
	reason: Option<Data<'a, Ascii>>,
	header_fields: HashMap<Data<'a, HeaderFieldKey>, Data<'a, Ascii>>
}
impl<'a> ResponseBuilder<'a> {
	/// Creates a new response builder
	pub fn new() -> Self {
		Self{ version: None, status: None, reason: None, header_fields: HashMap::new() }
	}
	
	/// Sets the HTTP version to the literal `version`
	pub fn version(mut self, version: Data<'a, Ascii>) -> Self {
		self.version = Some(version);
		self
	}
	/// Sets the status code
	pub fn status(mut self, status: Data<'a, Integer>) -> Self {
		self.status = Some(status);
		self
	}
	/// Sets the status reason
	pub fn reason(mut self, info: Data<'a, Ascii>) -> Self {
		self.reason = Some(info);
		self
	}
	
	/// Inserts a header field with `key`-`value`
	pub fn field(mut self, key: Data<'a, HeaderFieldKey>, value: Data<'a, Ascii>) -> Self {
		self.header_fields.insert(key, value);
		self
	}
	
	/// Builds the response header
	pub fn build(self) -> Result<ResponseHeader<'a>, HttpError> {
		// Unwrap status fields
		let version = self.version.ok_or(HttpError::ApiMisuse)?;
		let status = self.status.ok_or(HttpError::ApiMisuse)?;
		let reason = self.reason.ok_or(HttpError::ApiMisuse)?;
		
		Ok(ResponseHeader(Header {
			header_line: (version.as_slice(), status.as_slice(), reason.as_slice()),
			header_fields: self.header_fields
		}))
	}
}