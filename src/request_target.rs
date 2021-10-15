use crate::{
    error::Result,
    helpers::{ self, BufReadExt, MatchFlag::Trim }
};
use std::{
    mem, slice, collections::BTreeMap, iter::FromIterator, ops::Deref,
    io::{ BufRead, Write }
};


/// A request target (see <https://tools.ietf.org/html/rfc7230#section-5.3>)
///
/// ## Note
/// We only support the absolute and the asterisk form.
///
/// ## Warning
/// The path parser is pretty simple and basically follows two rules only:
///  - the path must either be a wildcard (`*`) or must begin with a slash (`/`)
///  - empty path components are ignored (i.e. `/`, `//` evaluate to `[]` and `/test//path`, `/test//path/` etc. evaluate
///    to `["test", "path"]`)
///
/// Other potentially dangerous components like `..` are treated like normal components and do __not__ cause an error.
///
/// ## Warning
/// The query parser is also pretty simple and basically parses any `key` or `key=` or `key=value` component without
/// further validation (with the excepttion that the URI escaping must be valid if present).
///
/// The following rules apply:
///  - the query string must begin with a `?`
///  - keys don't need a value (e.g. `?key0&key1`)
///  - keys can have an empty value (e.g. `?key0=&key1=`)
///  - keys can have a non-empty value (e.g. `?key0=value0&key1=value1`
///  - empty keys or key-value pairs are ignored (i.e. `?&` evaluates to `[]` or `?key0&&key1` evaluates to
///    `["key0": "", "key1": ""]` or `?=value0&key1=value1&` evaluates to `["key1": "value1"]`)
#[derive(Debug, Clone)]
pub enum RequestTarget {
    /// The requested path is a wildcard ("*" e.g. for `OPTIONS` requests)
    Wildcard,
    /// The requested path
    Absolute {
        /// The path components
        path: RequestTargetPath,
        /// The query string
        query: QueryString
    }
}
impl RequestTarget {
    /// Creates a new wildcard request target
    pub const fn new_wildcard() -> Self {
        Self::Wildcard
    }
    /// Creates a new request target from the given path and query components
    pub const fn new_absolute(path: RequestTargetPath, query: QueryString) -> Self {
        Self::Absolute { path, query }
    }
    /// Loads a request target
    pub fn read<T>(source: &mut T) -> Result<Self> where T: BufRead {
        // Read the URI
        let this = match source.peek_one()? {
            Some(b'/') => {
                // Read path and query
                let path = source.read_word("?", [Trim])?;
                let query = source.read_all([])?;
                
                // Init self
                Self::Absolute {
                    path: RequestTargetPath::read(&mut helpers::memreader(path))?,
                    query: QueryString::read(&mut helpers::memreader(query))?
                }
            },
            Some(b'*') => Self::Wildcard,
            first => Err(einval!("Invalid request target: {:?}", first))?
        };
        Ok(this)
    }
    /// Writes the request target
    pub fn write_all<T>(&self, output: &mut T) -> Result where T: Write {
        match self {
            Self::Absolute { path, query } => {
                path.write_all(output)?;
                query.write_all(output)?;
            },
            Self::Wildcard => write!(output, "*")?,
        }
        Ok(())
    }
}


/// An absolute path as request target
/// 
/// ## Warning
/// The path parser is pretty simple and basically follows two rules only:
///  - the path must either be a wildcard (`*`) or must begin with a slash (`/`)
///  - empty path components are ignored (i.e. `/` or `//` etc. evaluate to `[]` and `/test//path` or `/test//path/` etc.
///    evaluate to `["test", "path"]`)
///
/// Other potentially dangerous components like `..` are treated like normal components and __will not__ cause an error.
#[derive(Debug, Clone, Default)]
pub struct RequestTargetPath {
    /// The path components
    components: Vec<Vec<u8>>
}
impl RequestTargetPath {
    /// Creates a new empty absolute path
    pub const fn new() -> Self {
        Self { components: Vec::new() }
    }

    /// Pushes `component` to `self`
    pub fn push<T>(&mut self, component: T) where T: Into<Vec<u8>> {
        self.components.push(component.into());
    }

    /// Loads an absolute string
    pub fn read<T>(source: &mut T) -> Result<Self> where T: BufRead {
        // Create self
        let mut this = Self { components: Vec::new() };
        
        // Read the path components
        let mut component = Vec::new();
        while source.peek_one()?.is_some() {
            // Read the next byte
            let mut next = 0;
            source.read_exact(slice::from_mut(&mut next))?;

            // Push the next component
            match next {
                b'/' if component.is_empty() => (/* no-op */),
                b'/' => this.components.push(mem::take(&mut component)),
                other => component.push(other)
            }
        }
        Ok(this)
    }

    /// Writes the absolute path
    pub fn write_all<T>(&self, output: &mut T) -> Result where T: Write {
        // Write a single slash if there are no path components
        if self.components.is_empty() {
            output.write_all(b"/")?;
            return Ok(());
        }

        // Write the components
        for component in self.components.iter() {
            output.write_all(b"/")?;
            output.write_all(&component)?;
        }
        Ok(())
    }
}
impl<T> FromIterator<T> for RequestTargetPath where T: Into<Vec<u8>> {
    fn from_iter<I: IntoIterator<Item = T>>(components: I) -> Self {
        let components = components.into_iter()
            .map(|c| c.into())
            .collect();
        Self { components }
    }
}
impl IntoIterator for RequestTargetPath {
    type Item = <Vec<Vec<u8>> as IntoIterator>::Item;
    type IntoIter = <Vec<Vec<u8>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}


/// A query string
///
/// ## Warning
/// The query parser is pretty simple and basically parses any `key` or `key=` or `key=value` component without further
/// validation.
///
/// The following rules apply:
///  - the query string _MUST NOT_ begin with a `?` – it's not a bug, it's a feature: this allows the parser to parse query
///    query strings in the body (e.g. from HTML forms)
///  - keys don't need a value (i.e. `key0&key1` is valid)
///  - keys can have an empty value (i.e. `key0=&key1=` is valid)
///  - keys can have a non-empty value (i.e. `key0=value0&key1=value1` is valid)
///  - empty keys/key-value pairs are ignored (i.e. `&` evaluates to `[]`, `key0&&key1` evaluates to
///    `["key0": "", "key1": ""]` and `=value0&key1=value1&` evaluates to `["key1": "value1"]`)
#[derive(Debug, Clone, Default)]
pub struct QueryString {
    /// The key-value fields of the query string
    fields: BTreeMap<Vec<u8>, Vec<u8>>
}
impl QueryString {
    /// Creates a new header field map
    pub fn new() -> Self {
        Self { fields: BTreeMap::new() }
    }

    /// Gets the value for the field with the given name
    pub fn get<T>(&self, name: T) -> Option<&[u8]> where T: AsRef<[u8]> {
        self.fields.get(name.as_ref()).map(|s| s.as_ref())
    }
    /// Sets the value for a fiels with the given name
    pub fn set<A, B>(&mut self, name: A, value: B) where A: Into<Vec<u8>>, B: Into<Vec<u8>> {
        self.fields.insert(name.into(), value.into());
    }

    /// Reads a query string
    pub fn read<T>(source: &mut T) -> Result<Self> where T: BufRead {
        // Parse the query components
        let mut fields = BTreeMap::new();
        while source.peek_one()?.is_some() {
            // Read split the pair
            let pair = source.read_word("&", [Trim])?;
            let mut pair = helpers::memreader(pair);

            // Get key and value
            let key = pair.read_word("=", [Trim])?;
            if !key.is_empty() {
                let value = pair.read_all([])?;
                fields.insert(key, value);
            }
        }
        Ok(Self { fields })
    }
    /// Writes the query string
    pub fn write_all<T>(&self, output: &mut T) -> Result where T: Write {
        for (nth_pair, (key, value)) in self.fields.iter().enumerate() {
            // Write delimiter
            match nth_pair {
                0 => output.write_all(b"?")?,
                _ => output.write_all(b"&")?
            }

            // Write key and value
            output.write_all(&key)?;
            if !value.is_empty() {
                output.write_all(b"=")?;
                output.write_all(&value)?;
            }
        }
        Ok(())
    }
}
impl Deref for QueryString {
    type Target = BTreeMap<Vec<u8>, Vec<u8>>;
    
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
impl<K, V> FromIterator<(K, V)> for QueryString where K: Into<Vec<u8>>, V: Into<Vec<u8>> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(pairs: T) -> Self {
        let fields = pairs.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self { fields }
    }
}
impl IntoIterator for QueryString {
    type Item = <BTreeMap<Vec<u8>, Vec<u8>> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Vec<u8>, Vec<u8>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}