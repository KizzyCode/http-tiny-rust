mod helpers;

use http_header::{ Header, error::ErrorKind };
use std::{ collections::BTreeMap, ops::Deref };


struct Test {
    raw: &'static [u8],
    method: &'static [u8],
    target: &'static [u8],
    fields: BTreeMap<Vec<u8>, Vec<u8>>
}
impl Test {
    fn test(self) {
        let header = Header::read(&mut helpers::source(self.raw)).expect("Failed to read header");
        assert_eq!(self.method, header.start_line().request_method());
        assert_eq!(self.target, header.start_line().request_target());
        assert_eq!(&self.fields, header.fields().deref());
    }
}
#[test]
fn test() {
    Test {
        raw: b"HEAD / HTTP/1.1\r\n\r\n",
        method: b"HEAD",
        target: b"/",
        fields: BTreeMap::new(),
    }.test();
    
    Test {
        raw: concat!(
            "POST /upl%C3%B6ad/form.php HTTP/1.1\r\n",
            "Host: www.heise.de\r\n",
            "User-Agent: http_header/0.3.0\r\n",
            "\r\n",
            "Test\r\nBODY\r\nolope"
        ).as_bytes(),
        method: b"POST",
        target: b"/upl%C3%B6ad/form.php",
        fields: helpers::map([
            ("host", "www.heise.de"),
            ("user-agent", "http_header/0.3.0")
        ])
    }.test();
}


#[derive(Debug)]
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
        data: b"HEAD / HTTP/1.1\r\n",
        error: ErrorKind::InvalidValue
    }.test();
    TestErr {
        data: b"\r\n\r\n",
        error: ErrorKind::InvalidValue
    }.test();
    TestErr {
        data: b"HEAD / \r\n\r\n",
        error: ErrorKind::InvalidValue
    }.test();
    
    TestErr {
        data: concat!(
            "HEAD / HTTP/1.1\r\n",
            "Host: www.heise.de\r\n",
            "User-Agent \r\n",
            "\r\n"
        ).as_bytes(),
        error: ErrorKind::InvalidValue
    }.test();
}
