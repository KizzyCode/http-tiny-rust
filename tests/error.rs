use std::error;

#[macro_use]
extern crate http_tiny;

fn assert_impl_error<T>(object: T)
where
    T: error::Error,
{
    println!("Error: {object}");
}

#[test]
fn test() {
    let error = error!("Test error");
    assert_impl_error(error);
}
