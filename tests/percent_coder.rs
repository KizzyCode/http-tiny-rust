mod helpers;

use http_tiny::{ PercentEncoder, PercentDecoder };


#[derive(Debug)]
struct Test {
    raw: &'static [u8],
    expected: &'static [u8]
}
impl Test {
    pub fn test(self) {
        // Test encode
        let mut decoded = self.raw;
        let mut encoded = Vec::new();
        PercentEncoder::new().copy(&mut decoded, &mut encoded).expect("Failed to percent-encode bytes");
        assert_eq!(&encoded, self.expected, "Invalid percent-encoded bytes");

        // Test decode
        let mut encoded = self.expected;
        let mut decoded = Vec::new();
        PercentDecoder::new().copy(&mut encoded, &mut decoded).expect("Failed to percent-decode bytes");
        assert_eq!(&decoded, self.raw, "Invalid percent-decoded bytes");
    }
}
#[test]
fn test() {
    Test {
        raw: b"/Volumes/Data/\xF0\x9F\x8D\x86\x0A",
        expected: b"%2FVolumes%2FData%2F%F0%9F%8D%86%0A"
    }.test();
}
