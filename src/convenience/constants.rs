//! Some common constants

/// A `200` "OK" HTTP status code
pub const STATUS_200_OK: &str = "200";
/// A `400` "Bad Request" HTTP status code
pub const STATUS_400_BADREQUEST: &str = "400";
/// A `404` "Not Found" HTTP status code
pub const STATUS_404_NOTFOUND: &str = "404";
/// A `405` "Method Not Allowed" HTTP status code
pub const STATUS_405_METHODNOTALLOWED: &str = "405";
/// A `500` "Internal Server Error" HTTP status code
pub const STATUS_500_INTERNALSERVERERROR: &str = "500";

/// The header key for the "Content-Type" header field
pub const HEADER_CONTENTTYPE: &str = "content-type";
/// The header key for the "Content-Length" header field
pub const HEADER_CONTENTLENGTH: &str = "content-length";

/// Content type constant for "text/plain"
pub const CONTENTTYPE_TEXTPLAIN: &str = "text/plain";
/// Content type constant for "application/octet-stream"
pub const CONTENTTYPE_APPLICATIONOCTETSTREAM: &str = "application/octet-stream";
