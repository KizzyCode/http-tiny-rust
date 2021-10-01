mod helpers;

use http_header::{ Header, HeaderStartLine, HeaderFields };
use std::{ io::Cursor, iter::FromIterator };


struct Test {
    header: Header,
    raw: &'static [u8]
}
impl Test {
    pub fn test(self) {
        let mut serialized = Cursor::new(Vec::new());
        self.header.write(&mut serialized).unwrap();
        assert_eq!(self.raw, serialized.into_inner().as_slice())
    }
}
#[test]
fn test() {
    Test {
        header: Header::new(
            HeaderStartLine::new_request("GET", "/"),
            HeaderFields::new()
        ),
        raw: b"GET / HTTP/1.1\r\n\r\n"
    }.test();
    
    Test {
        header: Header::new(
            HeaderStartLine::new_request("GET", "/"),
            HeaderFields::from_iter([
                ("Host", "www.heise.de")
            ])
        ),
        raw: concat!(
            "GET / HTTP/1.1\r\n",
            "host: www.heise.de\r\n",
            "\r\n"
        ).as_bytes()
    }.test();
}
