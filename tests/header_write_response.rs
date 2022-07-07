mod helpers;

use http_tiny::{Header, HeaderFields, HeaderStartLine};
use std::{io::Cursor, iter::FromIterator};

struct Test {
    header: Header,
    raw: &'static [u8],
}
impl Test {
    pub fn test(self) {
        let mut serialized = Cursor::new(Vec::new());
        self.header.write_all(&mut serialized).unwrap();
        assert_eq!(self.raw, serialized.into_inner().as_slice())
    }
}
#[test]
fn test() {
    Test {
        header: Header::new(HeaderStartLine::new_response(200, "OK"), HeaderFields::new()),
        raw: b"HTTP/1.1 200 OK\r\n\r\n",
    }
    .test();

    Test {
        header: Header::new(
            HeaderStartLine::new_response(200, "OK"),
            HeaderFields::from_iter([("Date", "Sun, 26 May 2019 22:02:50 GMT")]),
        ),
        raw: concat!("HTTP/1.1 200 OK\r\n", "date: Sun, 26 May 2019 22:02:50 GMT\r\n", "\r\n").as_bytes(),
    }
    .test();
}
