//! Some encodings for the `Data` type

use crate::helpers::iter_ext::IterExt;
use std::{ str, fmt::Debug, hash::Hasher };


/// An encoding
pub trait Encoding: Copy + Clone + Debug + Default {
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
/// An encoding which is a subset of UTF-8
pub trait Utf8Subset: Encoding {}


/// Printable ASCII characters
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
#[derive(Copy, Clone, Debug, Default)]
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
impl Utf8Subset for Ascii {}


/// A header-field key according to [RFC 7230](https://tools.ietf.org/html/rfc7230#section-3.2)
///
/// This ASCII mode includes all
///  - alphabetic characters
///  - digits
///  - punctuation characters
///  - whitespace characters (U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED, U+000C FORM
///    FEED, or U+000D CARRIAGE RETURN)
#[derive(Copy, Clone, Debug, Default)]
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
impl Utf8Subset for HeaderFieldKey {}


/// A valid URI according to [RFC 3986](https://tools.ietf.org/html/rfc3986)
#[derive(Copy, Clone, Debug, Default)]
pub struct Uri;
impl Uri {
	/// Checks if `b` and the next two chars contain a valid
	/// [percent encoding](https://tools.ietf.org/html/rfc3986#section-2.1)
	fn percent_encoding(b: u8, next: &mut Iterator<Item = &u8>) -> bool {
		match b {
			b'%' => {
				// Take the next two bytes and ensure that they are hex digits
				let ab = match next.take(2).collect_min(2) {
					Some(ab) => ab,
					None => return false
				};
				ab.iter().all(|b| b.is_ascii_hexdigit())
			},
			_ => return false
		}
	}
	
	/// Checks if `b` is one of the
	/// [unreserved chars](https://tools.ietf.org/html/rfc3986#section-2.3)
	fn unreserved(b: u8) -> bool {
		b.is_ascii_alphanumeric() || b"-._~".contains(&b)
	}
	
	/// Checks if `b` is one of the [`gen-delims`](https://tools.ietf.org/html/rfc3986#section-2.2)
	fn gen_delims(b: u8) -> bool {
		b":/?#[]@".contains(&b)
	}
	/// Checks if `b` is one of the [`sub-delims`](https://tools.ietf.org/html/rfc3986#section-2.2)
	fn sub_delims(b: u8) -> bool {
		b"!$&'()*+,;=".contains(&b)
	}
	/// Checks if `b` is one of the
	/// [reserved chars](https://tools.ietf.org/html/rfc3986#section-2.2)
	fn reserved(b: u8) -> bool {
		Self::gen_delims(b) || Self::sub_delims(b)
	}
}
impl Encoding for Uri {
	fn is_valid(bytes: &[u8]) -> bool {
		// Validate that all characters are valid URI characters
		let mut bytes = bytes.iter();
		while let Some(b) = bytes.next() {
			match *b {
				b if Self::unreserved(b) => continue,
				b if Self::reserved(b) => continue,
				b => match Self::percent_encoding(b, &mut bytes) {
					true => continue,
					false => return false
				}
			}
		}
		true
	}
}
impl Utf8Subset for Uri {}


/// A valid query according to [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.4)
#[derive(Copy, Clone, Debug, Default)]
pub struct UriQuery;
impl Encoding for UriQuery {
	fn is_valid(bytes: &[u8]) -> bool {
		// Validate that all characters are valid URI characters
		let mut bytes = bytes.iter();
		while let Some(b) = bytes.next() {
			match *b {
				b if Uri::unreserved(b) => continue,
				b if Uri::sub_delims(b) => continue,
				b':' | b'@' => continue,
				b => match Uri::percent_encoding(b, &mut bytes) {
					true => continue,
					false => return false
				}
			}
		}
		true
	}
}
impl Utf8Subset for UriQuery {}


/// An ASCII encoded integer (U+0030 '0' ... U+0039 '9')
#[derive(Copy, Clone, Debug, Default)]
pub struct Integer;
impl Encoding for Integer {
	fn is_valid(bytes: &[u8]) -> bool {
		bytes.iter().all(u8::is_ascii_digit)
	}
}
impl Utf8Subset for Integer {}