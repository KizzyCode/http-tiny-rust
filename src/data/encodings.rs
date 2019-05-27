//! Some encodings for the `Data` type

use crate::helpers::iter_ext::IterExt;
use std::{ str, fmt::Debug, hash::Hasher };


/// An encoding
pub trait Encoding: Debug + Default {
	/// Checks if `bytes` conform to the encoding
	fn is_valid(bytes: &[u8]) -> bool;
	/// Checks if `a` is equal to `b` (can be overridden; e.g. to perform a case-insensitive
	/// comparison)
	fn is_eq(a: &[u8], b: &[u8]) -> bool {
		a == b
	}
	/// Hashes `bytes` (can be overridden; e.g. to compute a case-insensitive hash)
	fn hash(bytes: &[u8], hasher: &mut Hasher) {
		hasher.write(bytes);
	}
}
/// A human readable encoding
pub trait HumanReadable: Encoding {}


/// An encoding for printable ASCII characters
///
/// This ASCII mode includes all
///  - alphabetic characters (U+0041 'A' ... U+005A 'Z' or U+0061 'a' ... U+007A 'z')
///  - digits (U+0030 '0' ... U+0039 '9')
///  - whitespace characters (U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM
///    FEED, or U+000D CARRIAGE RETURN)
///  - punctuation characters:
///    - U+0021 ... U+002F `! " # $ % & ' ( ) * + , - . /`
///    - U+003A ... U+0040 `: ; < = > ? @`
///    - U+005B ... U+0060 ``[ \ ] ^ _ ` ``
///    - U+007B ... U+007E `{ | } ~`
#[derive(Debug, Default)]
pub struct Ascii;
impl Encoding for Ascii {
	fn is_valid(bytes: &[u8]) -> bool {
		fn is_printable(b: &u8) -> bool {
			b.is_ascii_alphanumeric()
				| b.is_ascii_whitespace()
				| b.is_ascii_punctuation()
		}
		bytes.iter().all(is_printable)
	}
}
impl HumanReadable for Ascii {}


/// A header-field-key according to
///
/// This ASCII mode includes all
///  - alphabetic characters
///  - digits
///  - punctuation characters
///  - whitespace characters (U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM
///    FEED, or U+000D CARRIAGE RETURN)
#[derive(Debug, Default)]
pub struct HeaderFieldKey;
impl Encoding for HeaderFieldKey {
	fn is_valid(bytes: &[u8]) -> bool {
		// Key must not be empty
		match bytes.len() {
			0 => false,
			_ => bytes.iter().all(|b| match *b {
				// Must be US-ASCII; control-chars or separators are invalid
				b if b > 127 => false,
				b if b < 32 => false,
				b if b"()<>@,;:/[]?={} \t\"\\".contains(&b) => false,
				_ => true
			})
		}
	}
	fn is_eq(a: &[u8], b: &[u8]) -> bool {
		let a= str::from_utf8(a).unwrap().to_ascii_lowercase();
		let b= str::from_utf8(b).unwrap().to_ascii_lowercase();
		a == b
	}
	fn hash(bytes: &[u8], hasher: &mut Hasher) {
		let s = str::from_utf8(bytes).unwrap().to_ascii_lowercase();
		hasher.write(s.as_bytes());
	}
}
impl HumanReadable for HeaderFieldKey {}


/// A valid URI according to [RFC 3986](https://tools.ietf.org/html/rfc3986#section-2)
#[derive(Debug, Default)]
pub struct Uri;
impl Encoding for Uri {
	fn is_valid(bytes: &[u8]) -> bool {
		// Validate that all characters are valid URI characters
		let mut bytes = bytes.iter();
		while let Some(b) = bytes.next() {
			match *b {
				// Unreserved chars
				b if b.is_ascii_alphanumeric() => continue,
				b if b"-._~".contains(&b) => continue,
				// Reserved chars
				b if b":/?#[]@".contains(&b) => continue,
				b if b"!$&'()*+,;=".contains(&b) => continue,
				// Percent encoding
				b'%' => {
					// Take the next two bytes
					let ab = match bytes.by_ref().take(2).collect_min(2) {
						Some(ab) => ab,
						None => return false
					};
					// Ensure that the next two bytes are hex-digits
					match ab.iter().all(|b| b.is_ascii_hexdigit()) {
						true => continue,
						false => return false
					}
				},
				_ => return false
			}
		}
		true
	}
}
impl HumanReadable for Uri {}


/// An ASCII encoded integer (U+0030 '0' ... U+0039 '9')
#[derive(Debug, Default)]
pub struct Integer;
impl Encoding for Integer {
	fn is_valid(bytes: &[u8]) -> bool {
		bytes.iter().all(u8::is_ascii_digit)
	}
}
impl HumanReadable for Integer {}