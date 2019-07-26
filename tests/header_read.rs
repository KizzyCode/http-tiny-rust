use http_header::Header;
use std::io::Cursor;


struct Test {
	data: &'static[u8],
	len: Option<usize>
}
impl Test {
	pub fn test(self) {
		let mut source = Cursor::new(self.data);
		
		let mut buf = [0; 8192];
		let len = Header::read(&mut source, &mut buf).unwrap();
		
		assert_eq!(self.len, len);
		match self.len {
			Some(len) => assert_eq!(source.position(), len as u64),
			None => assert_eq!(source.position(), self.data.len() as u64)
		}
	}
}
#[test]
fn test() {
	Test {
		data: b"POST /upl%C3%B6ad/form.php HTTP/1.1\r\n\r\nSome body data",
		len: Some(b"POST /upl%C3%B6ad/form.php HTTP/1.1\r\n\r\n".len())
	}.test();
	
	Test {
		data: b"POST /upl%C3%B6ad/form.php HTTP/1.1",
		len: None
	}.test();
}