use http_tiny::bytetraits::IntoBytes;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{BufReader, Cursor},
};

/// Creates a readable data source over a slice
#[allow(unused)]
pub fn source<T>(slice: T) -> BufReader<Cursor<T>>
where
    T: AsRef<[u8]>,
{
    BufReader::new(Cursor::new(slice))
}

/// Creates a new map
#[allow(unused)]
pub fn map<I, K, V>(pairs: I) -> BTreeMap<Cow<'static, [u8]>, Cow<'static, [u8]>>
where
    I: IntoIterator<Item = (K, V)>,
    K: IntoBytes,
    V: IntoBytes,
{
    pairs.into_iter().map(|(k, v)| (k.into_bytes(), v.into_bytes())).collect()
}
