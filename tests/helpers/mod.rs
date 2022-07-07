use std::{
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
pub fn map<I, K, V>(pairs: I) -> BTreeMap<Vec<u8>, Vec<u8>>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<Vec<u8>>,
    V: Into<Vec<u8>>,
{
    pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
}
