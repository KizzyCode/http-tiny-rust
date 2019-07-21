#[macro_use] extern crate http_header;
use http_header::{
	OwnedRef,
	header::{ RequestHeader, ResponseHeader },
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey }
	}
};
use std::{ collections::HashMap, pin::Pin };


macro_rules! map {
	($($key:expr => $value:expr),+) => ({
		let mut map = ::std::collections::HashMap::new();
		$( map.insert(data!($key), data!($value)); )*
		map
	});
	() => (::std::collections::HashMap::new());
}


struct ExpectedRequest {
	method: &'static str,
	uri: &'static str,
	version: &'static str,
	fields: HashMap<Data<'static, HeaderFieldKey>, Data<'static, Ascii>>
}
struct TestRequest {
	data: Vec<u8>,
	expected: ExpectedRequest
}
impl TestRequest {
	pub fn test(self) {
		let header = OwnedRef::<_, RequestHeader>::new(self.data).unwrap();
		
		Self::validate_ref(&header, &self.expected);
		Self::validate(header, self.expected);
	}
	
	fn validate_ref(header: &RequestHeader, expected: &ExpectedRequest) {
		assert_eq!(expected.method, header.method().unwrap());
		assert_eq!(expected.uri, header.uri().unwrap());
		assert_eq!(expected.version, header.version().unwrap());
		assert_eq!(&expected.fields, header.fields());
	}
	fn validate(header: Pin<Box<OwnedRef<Vec<u8>, RequestHeader>>>, expected: ExpectedRequest) {
		assert_eq!(expected.method, header.method().unwrap());
		assert_eq!(expected.uri, header.uri().unwrap());
		assert_eq!(expected.version, header.version().unwrap());
		assert_eq!(&expected.fields, header.fields());
	}
}


struct ExpectedResponse {
	version: &'static str,
	status: u16,
	reason: &'static str,
	fields: HashMap<Data<'static, HeaderFieldKey>, Data<'static, Ascii>>
}
struct TestResponse {
	data: Vec<u8>,
	expected: ExpectedResponse
}
impl TestResponse {
	pub fn test(self) {
		let header = OwnedRef::<_, ResponseHeader>::new(self.data).unwrap();
		Self::validate_ref(&header, &self.expected);
		Self::validate(header, self.expected);
	}
	
	fn validate_ref(header: &ResponseHeader, expected: &ExpectedResponse) {
		assert_eq!(expected.version, header.version().unwrap());
		assert_eq!(expected.status, header.status().unwrap());
		assert_eq!(expected.reason, header.reason().unwrap());
		assert_eq!(&expected.fields, header.fields());
	}
	fn validate(header: Pin<Box<OwnedRef<Vec<u8>, ResponseHeader>>>, expected: ExpectedResponse) {
		assert_eq!(expected.version, header.version().unwrap());
		assert_eq!(expected.status, header.status().unwrap());
		assert_eq!(expected.reason, header.reason().unwrap());
		assert_eq!(&expected.fields, header.fields());
	}
}


#[test]
fn test() {
	TestRequest {
		data: concat!(
			"POST /upl%C3%B6ad/form.php HTTP/1.1\r\n",
			"Host: www.heise.de\r\n",
			"User-Agent: http_header/0.3.0\r\n",
			"\r\n",
			"Test\r\nBODY\r\nolope"
		).as_bytes().to_vec(),
		expected: ExpectedRequest {
			method: "POST", uri: "/upl%C3%B6ad/form.php", version: "HTTP/1.1",
			fields: map!(
				"Host" => "www.heise.de",
				"User-Agent" => "http_header/0.3.0"
			)
		}
	}.test();
	
	TestResponse {
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
		).as_bytes().to_vec(),
		expected: ExpectedResponse {
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
			)
		}
	}.test();
}