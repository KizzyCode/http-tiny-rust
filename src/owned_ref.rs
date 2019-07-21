use crate::{
	HttpError,
	header::{ self, RequestHeader, ResponseHeader }
};
use std::{ marker::PhantomPinned, ops::Deref, pin::Pin, ptr::NonNull };


/// A helper trait for `OwnedRef`
pub trait OwnedRefHelper<'a, S, E>: Sized + 'a {
	/// Initializes `Self` with `source`
	fn init(source: &'a S) -> Result<Self, E>;
}
impl<'a> OwnedRefHelper<'a, Vec<u8>, HttpError> for RequestHeader<'a> {
	fn init(source: &'a Vec<u8>) -> Result<Self, HttpError> {
		Ok(header::parse_request(source)?.0)
	}
}
impl<'a> OwnedRefHelper<'a, Vec<u8>, HttpError> for ResponseHeader<'a> {
	fn init(source: &'a Vec<u8>) -> Result<Self, HttpError> {
		Ok(header::parse_response(source)?.0)
	}
}


/// A struct which combines an owned object and a reference type based upon the same object
pub struct OwnedRef<S, T> {
	source: S,
	pointer: NonNull<S>,
	target: Option<T>,
	_pin: PhantomPinned
}
impl<S, T> OwnedRef<S, T> {
	/// Creates a new owned reference with `source` as the underlying data segment
	pub fn new<'a, E>(source: S) -> Result<Pin<Box<Self>>, E> where S: 'a,
		T: OwnedRefHelper<'a, S, E>
	{
		// Create a base instance and pin it
		let this = Self {
			source, pointer: NonNull::dangling(),
			target: None, _pin: PhantomPinned
		};
		let mut pinned = Box::pin(this);
		
		// Create a pointer over the pinned source and inject it
		let pointer = NonNull::from(&pinned.source);
		unsafe {
			let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut pinned);
			Pin::get_unchecked_mut(mut_ref).pointer = pointer;
		}
		
		// Initialize target with the pointer to the pinned source and inject it
		let target = T::init(unsafe{ &*pointer.as_ptr() })?;
		unsafe {
			let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut pinned);
			Pin::get_unchecked_mut(mut_ref).target = Some(target);
		}
		
		Ok(pinned)
	}
}
impl<S, T> AsRef<T> for OwnedRef<S, T> {
	fn as_ref(&self) -> &T {
		self.target.as_ref().unwrap()
	}
}
impl<S, T> Deref for OwnedRef<S, T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.target.as_ref().unwrap()
	}
}