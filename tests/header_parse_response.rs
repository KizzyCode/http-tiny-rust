mod helpers;

use http_header::{ Header, error::ErrorKind };
use std::{ collections::BTreeMap, ops::Deref };


struct Test {
    raw: &'static [u8],
    status: u16,
    reason: &'static [u8],
    fields: BTreeMap<Vec<u8>, Vec<u8>>
}
impl Test {
    fn test(self) {
        let header = Header::read(&mut helpers::source(self.raw)).expect("Failed to read header");
        assert_eq!(self.status.to_string().as_bytes(), header.start_line().response_binstatus());
        assert_eq!(self.reason, header.start_line().response_reason());
        assert_eq!(&self.fields, header.fields().deref());
    }
}
#[test]
fn test() {
    Test {
        raw: b"HTTP/1.1 200 OK\r\n\r\n",
		status: 200,
		reason: b"OK",
        fields: BTreeMap::new()
    }.test();
    
    Test {
        raw: concat!(
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
		status: 200,
		reason: b"OK",
        fields: helpers::map([
            ("server", "nginx"),
            ("date", "Sun, 26 May 2019 22:02:50 GMT"),
            ("content-type", "text/html; charset=UTF-8"),
            ("last-modified", "Sun, 26 May 2019 22:02:50 GMT"),
            ("cache-control", "public, max-age=30"),
            ("age", "25"),
            ("strict-transport-security", "max-age=15768000"),
            ("x-frame-options", "DENY"),
            ("x-xss-protection", "1; mode=block"),
            ("x-content-type-options", "nosniff"),
            ("vary", "Accept-Encoding,X-Export-Format,X-Export-Agent"),
            ("accept-ranges", "bytes"),
            ("content-length", "417889"),
            ("connection", "keep-alive"),
		])
    }.test();
}


struct TestErr {
    data: &'static[u8],
    error: ErrorKind
}
impl TestErr {
    fn test(self) {
        let error = match Header::read(&mut helpers::source(self.data)) {
            Err(error) => error,
            Ok(header) => panic!("Unexpected `Ok` for header: {} ({:?})", String::from_utf8_lossy(self.data), header)
        };
        assert_eq!(&self.error, error.err(), "Unexpected error for header: {}", String::from_utf8_lossy(self.data));
    }
}
#[test]
fn test_err() {
    TestErr {
		data: b"HTTP/1.1 200 OK\r\n",
		error: ErrorKind::InvalidValue
	}.test();
    TestErr {
		data: b"\r\n\r\n",
		error: ErrorKind::InvalidValue
	}.test();
    TestErr {
		data: b"HTTP/1.1 200 \r\n\r\n",
		error: ErrorKind::InvalidValue
	}.test();
    
    TestErr {
        data: concat!(
            "HTTP/1.1 200 OK\r\n",
            "Server: nginx\r\n",
            "Date \r\n",
            "\r\n"
        ).as_bytes(),
		error: ErrorKind::InvalidValue
    }.test();
}
