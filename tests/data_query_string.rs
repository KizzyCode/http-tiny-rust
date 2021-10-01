mod helpers;

use http_header::RequestTarget;
use std::{ collections::BTreeMap, ops::Deref };


#[derive(Debug)]
struct Test {
    raw: &'static [u8],
    expected: BTreeMap<Vec<u8>, Vec<u8>>
}
impl Test {
    pub fn test(self) {
        let target = RequestTarget::read(&mut helpers::source(self.raw)).expect("Failed to read request target");
        let query = match target {
            RequestTarget::Absolute { query, .. } => query,
            target => panic!("Invalid request target for uri: {} ({:?})", String::from_utf8_lossy(self.raw), target)
        };
        assert_eq!(&self.expected, query.deref());
    }
}
#[test]
fn test() {
    Test {
        raw: b"/?code=M696be062-f150-bb19-9944-0c3a0ca60b48&state=99f4bd624dbe53d0ae330eabda904ac4",
        expected: helpers::map([
            ("code", "M696be062-f150-bb19-9944-0c3a0ca60b48"),
            ("state", "99f4bd624dbe53d0ae330eabda904ac4")
        ])
    }.test();
    Test {
        raw: concat!(
            "/secure.flickr.com/search/",
            "?q=tree+-swing&l=commderiv&d=taken-20000101-20051231&ct=0&lol&mt=all&adv=1&&"
        ).as_bytes(),
        expected: helpers::map([
            ("q", "tree+-swing"),
            ("l", "commderiv"),
            ("d", "taken-20000101-20051231"),
            ("ct", "0"),
            ("mt", "all"),
            ("adv", "1"),
            ("lol", "")
        ])
    }.test();
    
    Test {
        raw: b"/sth/?",
        expected: BTreeMap::new()
    }.test();
    Test {
        raw: b"/sth/",
        expected: BTreeMap::new()
    }.test();
}
