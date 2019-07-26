use crate::{
	HttpError, Header, RequestHeader, ResponseHeader,
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey, Uri }
	}
};
use std::{ collections::HashMap, convert::TryFrom };


/// A HTTP-response header builder
pub struct RequestBuilder {
	method: Option<Data<Ascii>>,
	uri: Option<Data<Uri>>,
	version: Option<Data<Ascii>>,
	header_fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>
}
impl RequestBuilder {
	/// Creates a new request builder
	pub fn new() -> Self {
		Self{ method: None, uri: None, version: None, header_fields: HashMap::new() }
	}
	
	/// Sets the request method
	pub fn method(mut self, method: Data<Ascii>) -> Self {
		self.method = Some(method);
		self
	}
	/// Sets the request URI
	pub fn uri(mut self, uri: Data<Uri>) -> Self {
		self.uri = Some(uri);
		self
	}
	/// Sets the HTTP version
	pub fn version(mut self, version: Data<Ascii>) -> Self {
		self.version = Some(version);
		self
	}
	
	/// Inserts a header field with `key`-`value`
	pub fn field(mut self, key: Data<HeaderFieldKey>, value: Data<Ascii>) -> Self {
		self.header_fields.insert(key, value);
		self
	}
	
	/// Builds the request header
	pub fn build(self) -> Result<RequestHeader, HttpError> {
		// Unwrap status fields
		let method = self.method.ok_or(HttpError::ApiMisuse)?;
		let uri = self.uri.ok_or(HttpError::ApiMisuse)?;
		let version = self.version.ok_or(HttpError::ApiMisuse)?;
		
		// Convert the URI into a generic ASCII field
		let uri_ascii: Data<Ascii> = Data::try_from(&uri as &[u8])?;
		Ok(RequestHeader {
			header: Header {
				status_line: (method, uri_ascii, version),
				fields: self.header_fields
			}, uri
		})
	}
}


/// A HTTP-response header builder
pub struct ResponseBuilder {
	version: Option<Data<Ascii>>,
	status: Option<u16>,
	reason: Option<Data<Ascii>>,
	header_fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>
}
impl ResponseBuilder {
	/// Creates a new response builder
	pub fn new() -> Self {
		Self{ version: None, status: None, reason: None, header_fields: HashMap::new() }
	}
	
	/// Sets the HTTP version to the literal `version`
	pub fn version(mut self, version: Data<Ascii>) -> Self {
		self.version = Some(version);
		self
	}
	/// Sets the status code
	pub fn status(mut self, status: u16) -> Self {
		self.status = Some(status);
		self
	}
	/// Sets the status reason
	pub fn reason(mut self, info: Data<Ascii>) -> Self {
		self.reason = Some(info);
		self
	}
	
	/// Inserts a header field with `key`-`value`
	pub fn field(mut self, key: Data<HeaderFieldKey>, value: Data<Ascii>) -> Self {
		self.header_fields.insert(key, value);
		self
	}
	
	/// Builds the response header
	pub fn build(self) -> Result<ResponseHeader, HttpError> {
		// Unwrap status fields
		let version = self.version.ok_or(HttpError::ApiMisuse)?;
		let status = self.status.ok_or(HttpError::ApiMisuse)?;
		let reason = self.reason.ok_or(HttpError::ApiMisuse)?;
		
		// Convert the status integer into a generic ASCII field
		let status_ascii: Data<Ascii> = Data::try_from(status.to_string().into_bytes())
			.expect("Should never fail because all number literals are a subset of ASCII");
		Ok(ResponseHeader {
			header: Header {
				status_line: (version, status_ascii, reason),
				fields: self.header_fields
			}, status
		})
	}
}