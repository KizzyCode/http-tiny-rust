//! Byte-conversion related traits

use std::borrow::Cow;

/// A trait for types that can be reasonably represented as a sequence of bytes without copying
pub trait AsBytes<'a>
where
    Self: Sized,
{
    /// Gets a zero-copy byte representation from `self`
    // Note: This trait is only implemented for refs, so this naming is semantically correct
    #[allow(clippy::wrong_self_convention)]
    fn as_bytes(self) -> &'a [u8];

    /// Converts slice to ASCII-lowercase (zero copy if the slice is already lowercase only)
    fn into_ascii_lowercase(self) -> Cow<'a, [u8]>
    where
        Self: 'a,
    {
        // Convert to lowercase if there is at least one uppercase char
        let this = self.as_bytes();
        match this.iter().any(|b| b.is_ascii_uppercase()) {
            true => Cow::Owned(this.to_ascii_lowercase()),
            false => Cow::Borrowed(this),
        }
    }
}
impl<'a> AsBytes<'a> for &'a str {
    fn as_bytes(self) -> &'a [u8] {
        str::as_bytes(self)
    }
}
impl<'a> AsBytes<'a> for &'a [u8] {
    fn as_bytes(self) -> &'a [u8] {
        self
    }
}

/// A trait for types that can be reasonably converted into a sequence of bytes
pub trait IntoBytes
where
    Self: Sized,
{
    /// Converts `self` into a byte representation (zero copy if possible)
    fn into_bytes(self) -> Cow<'static, [u8]>;

    /// Converts slice to ASCII-lowercase (zero copy if the slice is already lowercase only)
    fn into_ascii_lowercase(self) -> Cow<'static, [u8]> {
        // Check if we need a conversion
        let this = self.into_bytes();
        let needs_conversion = this.iter().any(|b| b.is_ascii_uppercase());

        // Convert to lowercase if there is at least one uppercase char
        match this {
            _ if !needs_conversion => this,
            Cow::Borrowed(_) => {
                // We need to alloc here
                Cow::Owned(this.to_ascii_lowercase())
            }
            Cow::Owned(mut bytes) => {
                // Convert in place to avoid allocation
                bytes.make_ascii_lowercase();
                Cow::Owned(bytes)
            }
        }
    }
}
impl IntoBytes for &'static str {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}
impl IntoBytes for String {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Owned(self.into_bytes())
    }
}
impl IntoBytes for &'static [u8] {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Borrowed(self)
    }
}
impl IntoBytes for Vec<u8> {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Owned(self)
    }
}
// Useful for content length fields etc.
impl IntoBytes for usize {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Owned(self.to_string().into_bytes())
    }
}
// Useful for content length fields or ranges etc.
impl IntoBytes for u64 {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Owned(self.to_string().into_bytes())
    }
}
// Useful for integer constants like response codes etc.
impl IntoBytes for i32 {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        Cow::Owned(self.to_string().into_bytes())
    }
}
