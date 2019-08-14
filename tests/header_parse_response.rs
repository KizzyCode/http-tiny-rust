#[macro_use] extern crate http_header;
use http_header::{
	HttpError, Header, ResponseHeader,
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
	version: &'static str,
	status: u16,
	reason: &'static str,
	fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>,
	body: &'static[u8]
}
impl Test {
	fn test(self) {
		let (header, body) = Header::scan(self.data).unwrap();
		let header: ResponseHeader = Header::parse(header).unwrap().try_into().unwrap();
		
		assert_eq!(self.version, header.version());
		assert_eq!(self.status, header.status());
		assert_eq!(self.reason, header.reason());
		assert_eq!(&self.fields, header.fields());
		assert_eq!(self.body, body);
	}
}
#[test]
fn test() {
	Test {
		data: b"HTTP/1.1 200 OK\r\n\r\n",
		version: "HTTP/1.1", status: 200, reason: "OK",
		fields: map!(),
		body: b""
	}.test();
	
	Test {
		data: concat!(
			"HTTP/1.1 200 OK\r\n",
			"Server: nginx\r\n",
			"Date: Sun, 26 May 2019 22:02:50 GMT\r\n",
			"Content-Type: text/html; charset=UTF-8\r\n",
			"Last-Modified: Sun, 26 May 2019 22:02:50 GMT\r\n",
			"Cache-Control: public, max-age=30\r\n",
			"Age: 25\r\n",
			"Strict-Transport-Security: max-age=15768000\r\n",
			"X-Frame-Options: DENY\r\n",
			"X-XSS-Protection: 1; mode=block\r\n",
			"X-Content-Type-Options: nosniff\r\n",
			"Vary: Accept-Encoding,X-Export-Format,X-Export-Agent\r\n",
			"Accept-Ranges: bytes\r\n",
			"Content-Length: 417889\r\n",
			"Connection: keep-alive\r\n",
			"\r\n",
			"Test\r\nBODY\r\nolope"
		).as_bytes(),
		version: "HTTP/1.1", status: 200, reason: "OK",
		fields: map!(
			"Server" => "nginx",
			"Date" => "Sun, 26 May 2019 22:02:50 GMT",
			"Content-Type" => "text/html; charset=UTF-8",
			"Last-Modified" => "Sun, 26 May 2019 22:02:50 GMT",
			"Cache-Control" => "public, max-age=30",
			"Age" => "25",
			"Strict-Transport-Security" => "max-age=15768000",
			"X-Frame-Options" => "DENY",
			"X-XSS-Protection" => "1; mode=block",
			"X-Content-Type-Options" => "nosniff",
			"Vary" => "Accept-Encoding,X-Export-Format,X-Export-Agent",
			"Accept-Ranges" => "bytes",
			"Content-Length" => "417889",
			"Connection" => "keep-alive"
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
			let _: ResponseHeader = Header::parse(data)?.try_into()?;
			Ok(())
		}
		assert_eq!(self.e, catch(self.data).unwrap_err())
	}
}
#[test]
fn test_err() {
	TestErr{ data: b"HTTP/1.1 200 OK\r\n", e: HttpError::TruncatedData }.test();
	TestErr{ data: b"\r\n\r\n", e: HttpError::ProtocolViolation }.test();
	TestErr{ data: b"HTTP/1.1 200 \r\n\r\n", e: HttpError::ProtocolViolation }.test();
	TestErr{ data: b"HTT\xC3\x9F/1.1 200 OK\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	TestErr{ data: b"HTTP/1.1 20O OK\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	TestErr{ data: b"HTTP/1.1 200 \xC3\x96K\r\n\r\n", e: HttpError::InvalidEncoding }.test();
	
	TestErr {
		data: concat!(
			"HTTP/1.1 200 OK\r\n",
			"Server: nginx\r\n",
			"Date \r\n",
			"\r\n"
		).as_bytes(), e: HttpError::ProtocolViolation
	}.test();
	
	TestErr {
		data: concat!(
			"HTTP/1.1 200 OK\r\n",
			"Server: nginx\r\n",
			"Date: Sun, 26 May 2019 22:02:50 GMT\r\n",
			"Content Type: text/html; charset=UTF-8\r\n",
			"\r\n"
		).as_bytes(), e: HttpError::InvalidEncoding
	}.test();
}