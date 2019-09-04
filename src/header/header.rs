use crate::{
	HttpError,
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey, Uri, Integer }
	},
	helpers::{
		iter_ext::IterExt,
		io_ext::{ ReadExt, WriteExt },
		slice_ext::{ ByteSliceExt, SliceExt }
	}
};
use std::{
	collections::HashMap,
	io::{ self, Read, Cursor },
	convert::{ TryFrom, TryInto }
};


/// A generic HTTP/1.* header implementation
///
/// Can be converted into a `RequestHeader`/`ResponseHeader` using the `TryFrom`/`TryInto`-traits
#[derive(Debug, Clone)]
pub struct Header {
	/// The header status line
	pub status_line: (Data<Ascii>, Data<Ascii>, Data<Ascii>),
	/// The header fields
	pub fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>
}
impl Header {
	/// Checks if `data` starts with a header-like structure and returns either
	/// `Some((header, body))` or `None` if no header-like structure was found
	pub fn scan(data: &[u8]) -> Option<(&[u8], &[u8])> {
		const END: &[u8] = b"\r\n\r\n";
		let index = data.find(&END)?;
		Some(data.split_at(index + END.len()))
	}
	/// Reads from `source` into `buf` until a HTTP-header-end is matched or `buf` is filled
	/// completely
	///
	/// Returns either `Some(header_len)` if the header end has been matched or `None` if `buf` has
	/// been filled completely without a match.
	pub fn read(mut source: impl Read, buf: &mut[u8]) -> Result<Option<usize>, io::Error> {
		const END: &[u8] = b"\r\n\r\n";
		source.read_until(buf, &END)
	}
	
	/// Parses a HTTP request header from `bytes`
	pub fn parse(bytes: &[u8]) -> Result<Self, HttpError> {
		const SPACE: &[u8] = b" ";
		const SEPARATOR: &[u8] = b":";
		const NEWLINE: &[u8] = b"\r\n";
		const END: &[u8] = b"\r\n\r\n";
		
		// Split data into header and body
		let header_body = bytes.splitn_pat(2, &END)
			.collect_min(2).ok_or(HttpError::TruncatedData)?;
		let mut header = header_body[0].split_pat(&NEWLINE);
		
		// Parse status line
		let status_line = header.next().ok_or(HttpError::ProtocolViolation)?
			.trim().splitn_pat(3, &SPACE)
			.collect_exact(3).ok_or(HttpError::ProtocolViolation)?;
		let status_line = (
			status_line[0].try_into()?,
			status_line[1].try_into()?,
			status_line[2].try_into()?
		);
		
		// Parse header fields
		let mut fields = HashMap::new();
		while let Some(line) = header.next() {
			let key_value = line.splitn_pat(2, &SEPARATOR)
				.collect_min(2).ok_or(HttpError::ProtocolViolation)?;
			fields.insert(
				Data::try_from(key_value[0])?,
				Data::try_from(key_value[1].trim())?
			);
		}
		Ok(Self{ status_line, fields })
	}
	
	/// Serializes and writes the header to `sink` and returns the amount of bytes written
	pub fn write(&self, mut sink: impl WriteExt) -> Result<usize, io::Error> {
		const SPACE: &[u8] = b" ";
		const SEPARATOR: &[u8] = b": ";
		const NEWLINE: &[u8] = b"\r\n";
		let mut written = 0;
		
		// Write header line
		sink.write(&self.status_line.0)?.write(SPACE)?
			.write(&self.status_line.1)?.write(SPACE)?
			.write(&self.status_line.2)?.write(NEWLINE)?;
		written += self.status_line.0.len() + SPACE.len()
			+ self.status_line.1.len() + SPACE.len()
			+ self.status_line.2.len() + NEWLINE.len();
		
		// Write header fields
		for (k, v) in self.fields.iter() {
			sink.write(k)?.write(SEPARATOR)?.write(v)?.write(NEWLINE)?;
			written += k.len() + SEPARATOR.len() + v.len() + NEWLINE.len();
		}
		
		// Write trailing newline
		sink.write(NEWLINE)?;
		written += NEWLINE.len();
		Ok(written)
	}
	/// Serializes the header into a vector
	pub fn to_vec(&self) -> Vec<u8> {
		let mut sink = Cursor::new(Vec::new());
		self.write(&mut sink).unwrap();
		sink.into_inner()
	}
}


/// Implements repetitive header functions
macro_rules! repetitive_header_fns {
	($struct:ty) => {
		impl $struct {
			/// Gets the field for `key` if any
			pub fn field(&self, key: &Data<HeaderFieldKey>) -> Option<&Data<Ascii>> {
				self.header.fields.get(&key)
			}
			/// Returns an iterator over all header fields
			pub fn fields(&self) -> &HashMap<Data<HeaderFieldKey>, Data<Ascii>> {
				&self.header.fields
			}
			
			/// Serializes and writes the header to `sink` and returns the amount of bytes written
			pub fn write(&self, sink: impl WriteExt) -> Result<usize, io::Error> {
				self.header.write(sink)
			}
			/// Serializes the header into a vector
			pub fn to_vec(&self) -> Vec<u8> {
				self.header.to_vec()
			}
		}
		impl AsRef<Header> for $struct {
			fn as_ref(&self) -> &Header {
				&self.header
			}
		}
		impl From<$struct> for Header {
			fn from(s: $struct) -> Self {
				s.header
			}
		}
	};
}


/// A HTTP request header
#[derive(Debug, Clone)]
pub struct RequestHeader{
	pub(in crate::header) header: Header,
	pub(in crate::header) uri: Data<Uri>
}
impl RequestHeader {
	/// The request method
	pub fn method(&self) -> &Data<Ascii> {
		&self.header.status_line.0
	}
	/// The requested URI
	pub fn uri(&self) -> &Data<Uri> {
		&self.uri
	}
	/// The HTTP version
	pub fn version(&self) -> &Data<Ascii> {
		&self.header.status_line.2
	}
}
repetitive_header_fns!(RequestHeader);
impl TryFrom<Header> for RequestHeader {
	type Error = HttpError;
	/// Tries to create a `RequestHeader` from a `Header`
	fn try_from(header: Header) -> Result<Self, Self::Error> {
		let uri = Data::try_from(&header.status_line.1 as &[u8])?;
		Ok(Self{ header, uri })
	}
}


/// A HTTP response header
#[derive(Debug, Clone)]
pub struct ResponseHeader {
	pub(in crate::header) header: Header,
	pub(in crate::header) status: u16
}
impl ResponseHeader {
	/// The HTTP version
	pub fn version(&self) -> &Data<Ascii> {
		&self.header.status_line.0
	}
	/// The status code
	pub fn status(&self) -> u16 {
		self.status
	}
	/// The status reason
	pub fn reason(&self) -> &Data<Ascii> {
		&self.header.status_line.2
	}
}
repetitive_header_fns!(ResponseHeader);
impl TryFrom<Header> for ResponseHeader {
	type Error = HttpError;
	/// Tries to create a `RequestHeader` from a `Header`
	fn try_from(header: Header) -> Result<Self, Self::Error> {
		let status_data: Data<Integer> = Data::try_from(&header.status_line.1 as &[u8])?;
		let status = status_data.try_into().map_err(|_| HttpError::ProtocolViolation)?;
		Ok(Self{ header, status })
	}
}