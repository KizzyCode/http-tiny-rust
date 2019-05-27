#[macro_use] extern crate http_header;
use http_header::{
	HttpError,
	header::{ RequestBuilder, RequestHeader }
};
use std::io::Cursor;


struct Test {
	header: RequestHeader<'static>,
	data: &'static[u8]
}
impl Test {
	pub fn test(self) {
		let mut serialized = Cursor::new(Vec::new());
		self.header.write(&mut serialized).unwrap();
		assert_eq!(self.data, serialized.into_inner().as_slice())
	}
}
#[test]
fn test() {
	Test {
		header: RequestBuilder::new()
			.method(data!("GET"))
			.uri(data!("/"))
			.version(data!("HTTP/1.1"))
			.build().unwrap(),
		data: b"GET / HTTP/1.1\r\n\r\n"
	}.test();
	
	Test {
		header: RequestBuilder::new()
			.method(data!("GET"))
			.uri(data!("/"))
			.version(data!("HTTP/1.1"))
			.field(data!("Host"), data!("www.heise.de"))
			.build().unwrap(),
		data: concat!(
			"GET / HTTP/1.1\r\n",
			"Host: www.heise.de\r\n",
			"\r\n"
		).as_bytes()
	}.test();
}


struct TestErr {
	builder: RequestBuilder<'static>,
	e: HttpError
}
impl TestErr {
	pub fn test(self) {
		let e = self.builder.build().unwrap_err();
		assert_eq!(self.e, e)
	}
}
#[test]
fn test_err() {
	TestErr {
		builder: RequestBuilder::new()
			.method(data!("GET"))
			.uri(data!("/")),
		e: HttpError::ApiMisuse
	}.test();
	
	TestErr {
		builder: RequestBuilder::new()
			.method(data!("GET"))
			.version(data!("HTTP/1.1")),
		e: HttpError::ApiMisuse
	}.test();
	
	TestErr {
		builder: RequestBuilder::new()
			.uri(data!("/"))
			.version(data!("HTTP/1.1")),
		e: HttpError::ApiMisuse
	}.test();
}