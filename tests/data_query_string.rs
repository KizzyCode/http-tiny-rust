#[macro_use] extern crate http_header;
use http_header::{
	QueryString,
	data::{
		Data,
		encodings::{ Uri, UriQuery }
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
	uri: &'static[u8],
	expected: HashMap<Data<'static, UriQuery>, Data<'static, UriQuery>>
}
impl Test {
	pub fn test(self) {
		let uri: Data<'static, Uri> = self.uri.try_into().unwrap();
		let query: QueryString<'static> = uri.try_into().unwrap();
		assert_eq!(&self.expected, query.fields());
	}
}
#[test]
fn test() {
	Test {
		uri: b"/?code=M696be062-f150-bb19-9944-0c3a0ca60b48&state=99f4bd624dbe53d0ae330eabda904ac4",
		expected: map!(
			"code" => "M696be062-f150-bb19-9944-0c3a0ca60b48",
			"state" => "99f4bd624dbe53d0ae330eabda904ac4"
		)
	}.test();
	
	Test {
		uri: concat!(
			"/secure.flickr.com/search/",
			"?q=tree+-swing&l=commderiv&d=taken-20000101-20051231&ct=0&lol&mt=all&adv=1&&"
		).as_bytes(),
		expected: map!(
			"q" => "tree+-swing",
			"l" => "commderiv",
			"d" => "taken-20000101-20051231",
			"ct" => "0",
			"mt" => "all",
			"adv" => "1",
			"lol" => ""
		)
	}.test();
	
	Test{ uri: b"/sth/?", expected: map!() }.test();
	Test{ uri: b"/sth/", expected: map!() }.test();
}


