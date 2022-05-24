#[macro_use] extern crate http_tiny;

use std::error::Error;


fn assert_impl_error<T>(object: T) where T: Error {
    println!("Error: {object}");
}


#[test]
fn test() {
    let error = eio!("Some test error");
    assert_impl_error(error);
}
