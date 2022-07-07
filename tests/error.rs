extern crate http_tiny;

use http_tiny::error::Error;

fn assert_impl_error<T>(object: T)
where
    T: std::error::Error,
{
    println!("Error: {object}");
}

#[test]
fn test() {
    let error = Error::Http;
    assert_impl_error(error);
}
