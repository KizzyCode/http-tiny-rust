#[macro_use] extern crate http_header;
use http_header::{ HttpError, ResponseBuilder, ResponseHeader };
use std::io::Cursor;


struct Test {
	header: ResponseHeader,
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
		header: ResponseBuilder::new()
			.version(data!("HTTP/1.1"))
			.status(200)
			.reason(data!("OK"))
			.build().unwrap(),
		data: b"HTTP/1.1 200 OK\r\n\r\n"
	}.test();
	
	Test {
		header: ResponseBuilder::new()
			.version(data!("HTTP/1.1"))
			.status(200)
			.reason(data!("OK"))
			.field(data!("Date"), data!("Sun, 26 May 2019 22:02:50 GMT"))
			.build().unwrap(),
		data: concat!(
			"HTTP/1.1 200 OK\r\n",
			"Date: Sun, 26 May 2019 22:02:50 GMT\r\n",
			"\r\n"
		).as_bytes()
	}.test();
}


struct TestErr {
	builder: ResponseBuilder,
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
		builder: ResponseBuilder::new()
			.version(data!("HTTP/1.1"))
			.status(200),
		e: HttpError::ApiMisuse
	}.test();
	
	TestErr {
		builder: ResponseBuilder::new()
			.version(data!("HTTP/1.1"))
			.reason(data!("OK")),
		e: HttpError::ApiMisuse
	}.test();
	
	TestErr {
		builder: ResponseBuilder::new()
			.status(200)
			.reason(data!("OK")),
		e: HttpError::ApiMisuse
	}.test();
}