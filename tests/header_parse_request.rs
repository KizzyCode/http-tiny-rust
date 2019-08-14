#[macro_use] extern crate http_header;
use http_header::{
	HttpError, Header, RequestHeader,
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey }
	}
};
use std::{ collections::HashMap, convert::TryInto };


macro_rules! map {
	($($key:expr => $value:expr),+) => ({
		let mut map = ::std::collections::HashMap::new();
		$( map.insert(data!($key), data!($value)); )*
		map
	});
	() => (::std::collections::HashMap::new());
}


struct Test {
	data: &'static[u8],
	method: &'static str,
	uri: &'static str,
	version: &'static str,
	fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>,
	body: &'static[u8]
}
impl Test {
	fn test(self) {
		let (header, body) = Header::scan(self.data).unwrap();
		let header: RequestHeader = Header::parse(header).unwrap().try_into().unwrap();
		
		assert_eq!(self.method, header.method());
		assert_eq!(self.uri, header.uri());
		assert_eq!(self.version, header.version());
		assert_eq!(&self.fields, header.fields());
		assert_eq!(self.body, body);
	}
}
#[test]
fn test() {
	Test {
		data: b"HEAD / HTTP/1.1\r\n\r\n",
		method: "HEAD", uri: "/", version: "HTTP/1.1",
		fields: map!(),
		body: b""
	}.test();
	
	Test {
		data: concat!(
			"POST /upl%C3%B6ad/form.php HTTP/1.1\r\n",
			"Host: www.heise.de\r\n",
			"User-Agent: http_header/0.3.0\r\n",
			"\r\n",
			"Test\r\nBODY\r\nolope"
		).as_bytes(),
		method: "POST", uri: "/upl%C3%B6ad/form.php", version: "HTTP/1.1",
		fields: map!(
			"Host" => "www.heise.de",
			"User-Agent" => "http_header/0.3.0"
		),
		body: b"Test\r\nBODY\r\nolope"
	}.test();
}


struct TestErr {
	data: &'static[u8],
	e: HttpError
}
impl TestErr {
	fn test(self) {
		fn catch(data: &'static[u8]) -> Result<(), HttpError> {
			let _: RequestHeader = Header::parse(data)?.try_into()?;
			Ok(())
		}
		assert_eq!(self.e, catch(self.data).unwrap_err())
	}
}
#[test]
fn test_err() {
	TestErr{ data: b"HEAD / HTTP/1.1\r\n", e: HttpError::TruncatedData }.test();
	TestErr{ data: b"\r\n\r\n", e: HttpError::ProtocolViolation }.test();
	TestErr{ data: b"HEAD / \r\n\r\n", e: HttpError::ProtocolViolation }.test();
	TestErr{ data: b"H\xC3\xA4D / HTTP/1.1\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	TestErr{ data: b"HEAD /l\xC3\xB6l HTTP/1.1\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	TestErr{ data: b"HEAD / HTT\xC3\x9F/1.1\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	
	TestErr {
		data: concat!(
			"HEAD / HTTP/1.1\r\n",
			"Host: www.heise.de\r\n",
			"User-Agent \r\n",
			"\r\n"
		).as_bytes(), e: HttpError::ProtocolViolation
	}.test();
	
	TestErr {
		data: concat!(
			"HEAD / HTTP/1.1\r\n",
			"Host: www.heise.de\r\n",
			"User Agent: http_header/0.3.0\r\n",
			"\r\n"
		).as_bytes(), e: HttpError::InvalidEncoding
	}.test();
}