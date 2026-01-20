//! Path and Query Parameter Parsing
//!
//! This module provides utilities for parsing and extracting parameters from URLs.
//! It handles both path parameters (`:id`, `:slug`) and query parameters (`?page=1`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur during parameter parsing.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParamsError {
    /// The parameter was not found in the URL.
    #[error("parameter '{0}' not found")]
    NotFound(String),

    /// The parameter value could not be parsed to the requested type.
    #[error("failed to parse parameter '{0}': {1}")]
    ParseError(String, String),

    /// The URL query string is malformed.
    #[error("malformed query string: {0}")]
    MalformedQuery(String),
}

/// Result type for parameter operations.
pub type ParamsResult<T> = Result<T, ParamsError>;

/// A collection of path parameters extracted from a URL.
///
/// Path parameters are defined in route patterns using the `:param` syntax.
/// For example, `/users/:id` will extract `id` from URLs like `/users/123`.
///
/// # Example
///
/// ```rust
/// use oxide_router::params::PathParams;
///
/// let mut params = PathParams::new();
/// params.insert("id", "123");
/// params.insert("slug", "hello-world");
///
/// assert_eq!(params.get("id"), Some("123"));
/// assert_eq!(params.get::<i32>("id").unwrap(), 123);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathParams {
    params: HashMap<String, String>,
}

impl PathParams {
    /// Creates a new empty `PathParams` collection.
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Inserts a path parameter with the given name and value.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.params.insert(name.into(), value.into());
    }

    /// Returns the raw string value of a parameter, if it exists.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Returns the parameter value parsed as the specified type.
    ///
    /// # Errors
    ///
    /// Returns `ParamsError::NotFound` if the parameter doesn't exist.
    /// Returns `ParamsError::ParseError` if the value can't be parsed.
    pub fn get_as<T: FromStr>(&self, name: &str) -> ParamsResult<T> {
        let value = self
            .params
            .get(name)
            .ok_or_else(|| ParamsError::NotFound(name.to_string()))?;

        value.parse::<T>().map_err(|_| {
            ParamsError::ParseError(
                name.to_string(),
                format!("cannot parse '{}' as {}", value, std::any::type_name::<T>()),
            )
        })
    }

    /// Returns the parameter value or a default if not found or unparseable.
    pub fn get_or<T: FromStr>(&self, name: &str, default: T) -> T {
        self.get_as(name).unwrap_or(default)
    }

    /// Returns an iterator over all parameter name-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Returns the number of path parameters.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Returns true if there are no path parameters.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Checks if a parameter with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }
}

impl From<HashMap<String, String>> for PathParams {
    fn from(params: HashMap<String, String>) -> Self {
        Self { params }
    }
}

/// A collection of query parameters extracted from a URL.
///
/// Query parameters appear after the `?` in a URL and are separated by `&`.
/// For example, `?page=1&sort=name&filter=active`.
///
/// # Example
///
/// ```rust
/// use oxide_router::params::QueryParams;
///
/// let params = QueryParams::parse("page=1&sort=name&tags=a&tags=b").unwrap();
///
/// assert_eq!(params.get("page"), Some("1"));
/// assert_eq!(params.get::<i32>("page").unwrap(), 1);
/// assert_eq!(params.get_all("tags"), vec!["a", "b"]);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    /// Creates a new empty `QueryParams` collection.
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Parses query parameters from a query string (without the leading `?`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_router::params::QueryParams;
    ///
    /// let params = QueryParams::parse("name=john&age=30").unwrap();
    /// assert_eq!(params.get("name"), Some("john"));
    /// ```
    pub fn parse(query: &str) -> ParamsResult<Self> {
        let mut params = HashMap::new();

        if query.is_empty() {
            return Ok(Self { params });
        }

        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }

            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };

            // URL decode the key and value
            let key = Self::url_decode(key).map_err(|e| ParamsError::MalformedQuery(e))?;
            let value = Self::url_decode(value).map_err(|e| ParamsError::MalformedQuery(e))?;

            params
                .entry(key)
                .or_insert_with(Vec::new)
                .push(value);
        }

        Ok(Self { params })
    }

    /// Simple URL decoding (percent decoding).
    fn url_decode(input: &str) -> Result<String, String> {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() != 2 {
                        return Err(format!("incomplete percent encoding in '{}'", input));
                    }
                    match u8::from_str_radix(&hex, 16) {
                        Ok(byte) => result.push(byte as char),
                        Err(_) => {
                            return Err(format!("invalid percent encoding '%{}'", hex));
                        }
                    }
                }
                '+' => result.push(' '),
                _ => result.push(c),
            }
        }

        Ok(result)
    }

    /// URL encodes a string for use in query parameters.
    pub fn url_encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 3);

        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => result.push('+'),
                _ => {
                    result.push('%');
                    result.push_str(&format!("{:02X}", byte));
                }
            }
        }

        result
    }

    /// Inserts a query parameter (can be called multiple times for the same key).
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.params
            .entry(name.into())
            .or_insert_with(Vec::new)
            .push(value.into());
    }

    /// Sets a query parameter, replacing any existing values.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.params.insert(name.into(), vec![value.into()]);
    }

    /// Returns the first value for a parameter, if it exists.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params.get(name).and_then(|v| v.first()).map(|s| s.as_str())
    }

    /// Returns the first value parsed as the specified type.
    pub fn get_as<T: FromStr>(&self, name: &str) -> ParamsResult<T> {
        let value = self
            .get(name)
            .ok_or_else(|| ParamsError::NotFound(name.to_string()))?;

        value.parse::<T>().map_err(|_| {
            ParamsError::ParseError(
                name.to_string(),
                format!("cannot parse '{}' as {}", value, std::any::type_name::<T>()),
            )
        })
    }

    /// Returns the first value or a default if not found or unparseable.
    pub fn get_or<T: FromStr>(&self, name: &str, default: T) -> T {
        self.get_as(name).unwrap_or(default)
    }

    /// Returns all values for a parameter (for multi-value parameters).
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.params
            .get(name)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Removes a parameter and returns all its values.
    pub fn remove(&mut self, name: &str) -> Option<Vec<String>> {
        self.params.remove(name)
    }

    /// Returns an iterator over all parameter names and their first values.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params
            .iter()
            .filter_map(|(k, v)| v.first().map(|first| (k.as_str(), first.as_str())))
    }

    /// Returns an iterator over all parameter names and all their values.
    pub fn iter_all(&self) -> impl Iterator<Item = (&str, &[String])> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    /// Returns the number of unique parameter names.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Returns true if there are no query parameters.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Checks if a parameter with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    /// Converts the query parameters back to a query string.
    pub fn to_query_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        for (key, values) in &self.params {
            for value in values {
                let encoded_key = Self::url_encode(key);
                let encoded_value = Self::url_encode(value);
                parts.push(format!("{}={}", encoded_key, encoded_value));
            }
        }

        parts.join("&")
    }
}

/// A parsed URL path with extracted segments.
///
/// This structure represents a URL path broken down into its components
/// for easier manipulation and comparison with route patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedPath {
    /// The original path string.
    pub raw: String,
    /// The individual path segments (split by '/').
    pub segments: Vec<String>,
    /// Whether the path ends with a trailing slash.
    pub trailing_slash: bool,
}

impl ParsedPath {
    /// Parses a path string into its components.
    pub fn parse(path: &str) -> Self {
        let path = if path.is_empty() { "/" } else { path };
        let raw = path.to_string();
        let trailing_slash = path.ends_with('/') && path.len() > 1;

        let segments: Vec<String> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Self {
            raw,
            segments,
            trailing_slash,
        }
    }

    /// Returns the number of segments in the path.
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Returns a new ParsedPath representing the parent path.
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            return None;
        }

        let segments = self.segments[..self.segments.len() - 1].to_vec();
        let raw = if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        };

        Some(Self {
            raw,
            segments,
            trailing_slash: false,
        })
    }

    /// Joins this path with another path segment.
    pub fn join(&self, segment: &str) -> Self {
        let mut segments = self.segments.clone();

        for s in segment.split('/').filter(|s| !s.is_empty()) {
            if s == ".." {
                segments.pop();
            } else if s != "." {
                segments.push(s.to_string());
            }
        }

        let raw = if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        };

        Self {
            raw,
            segments,
            trailing_slash: segment.ends_with('/'),
        }
    }
}

impl std::fmt::Display for ParsedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// A complete parsed URL with path, query parameters, and optional fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedUrl {
    /// The full original URL string.
    pub raw: String,
    /// The parsed path component.
    pub path: ParsedPath,
    /// The query parameters.
    pub query: QueryParams,
    /// The fragment (hash) if present, without the leading '#'.
    pub fragment: Option<String>,
}

impl ParsedUrl {
    /// Parses a URL string into its components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_router::params::ParsedUrl;
    ///
    /// let url = ParsedUrl::parse("/users/123?sort=name#section1").unwrap();
    /// assert_eq!(url.path.raw, "/users/123");
    /// assert_eq!(url.query.get("sort"), Some("name"));
    /// assert_eq!(url.fragment, Some("section1".to_string()));
    /// ```
    pub fn parse(url: &str) -> ParamsResult<Self> {
        let raw = url.to_string();

        // Split off fragment
        let (url_without_fragment, fragment) = match url.split_once('#') {
            Some((path, frag)) => (path, Some(frag.to_string())),
            None => (url, None),
        };

        // Split path and query
        let (path_str, query_str) = match url_without_fragment.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (url_without_fragment, None),
        };

        let path = ParsedPath::parse(path_str);
        let query = match query_str {
            Some(q) => QueryParams::parse(q)?,
            None => QueryParams::new(),
        };

        Ok(Self {
            raw,
            path,
            query,
            fragment,
        })
    }

    /// Reconstructs the URL string from its components.
    pub fn to_string(&self) -> String {
        let mut result = self.path.raw.clone();

        if !self.query.is_empty() {
            result.push('?');
            result.push_str(&self.query.to_query_string());
        }

        if let Some(ref fragment) = self.fragment {
            result.push('#');
            result.push_str(fragment);
        }

        result
    }

    /// Returns just the path portion of the URL.
    pub fn path_string(&self) -> &str {
        &self.path.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod path_params {
        use super::*;

        #[test]
        fn test_new_empty() {
            let params = PathParams::new();
            assert!(params.is_empty());
            assert_eq!(params.len(), 0);
        }

        #[test]
        fn test_insert_and_get() {
            let mut params = PathParams::new();
            params.insert("id", "123");
            params.insert("slug", "hello-world");

            assert_eq!(params.get("id"), Some("123"));
            assert_eq!(params.get("slug"), Some("hello-world"));
            assert_eq!(params.get("missing"), None);
        }

        #[test]
        fn test_get_as_types() {
            let mut params = PathParams::new();
            params.insert("int", "42");
            params.insert("float", "3.14");
            params.insert("bool", "true");
            params.insert("invalid", "not-a-number");

            assert_eq!(params.get_as::<i32>("int").unwrap(), 42);
            assert_eq!(params.get_as::<f64>("float").unwrap(), 3.14);
            assert_eq!(params.get_as::<bool>("bool").unwrap(), true);

            assert!(matches!(
                params.get_as::<i32>("invalid"),
                Err(ParamsError::ParseError(_, _))
            ));
            assert!(matches!(
                params.get_as::<i32>("missing"),
                Err(ParamsError::NotFound(_))
            ));
        }

        #[test]
        fn test_get_or_default() {
            let mut params = PathParams::new();
            params.insert("page", "5");

            assert_eq!(params.get_or("page", 1), 5);
            assert_eq!(params.get_or("missing", 1), 1);
        }

        #[test]
        fn test_contains() {
            let mut params = PathParams::new();
            params.insert("id", "123");

            assert!(params.contains("id"));
            assert!(!params.contains("missing"));
        }

        #[test]
        fn test_iter() {
            let mut params = PathParams::new();
            params.insert("a", "1");
            params.insert("b", "2");

            let collected: HashMap<&str, &str> = params.iter().collect();
            assert_eq!(collected.len(), 2);
            assert_eq!(collected.get("a"), Some(&"1"));
            assert_eq!(collected.get("b"), Some(&"2"));
        }
    }

    mod query_params {
        use super::*;

        #[test]
        fn test_parse_simple() {
            let params = QueryParams::parse("name=john&age=30").unwrap();

            assert_eq!(params.get("name"), Some("john"));
            assert_eq!(params.get("age"), Some("30"));
            assert_eq!(params.len(), 2);
        }

        #[test]
        fn test_parse_empty() {
            let params = QueryParams::parse("").unwrap();
            assert!(params.is_empty());
        }

        #[test]
        fn test_parse_no_value() {
            let params = QueryParams::parse("flag").unwrap();
            assert_eq!(params.get("flag"), Some(""));
        }

        #[test]
        fn test_parse_multi_value() {
            let params = QueryParams::parse("tags=a&tags=b&tags=c").unwrap();

            assert_eq!(params.get("tags"), Some("a"));
            assert_eq!(params.get_all("tags"), vec!["a", "b", "c"]);
        }

        #[test]
        fn test_parse_url_encoded() {
            let params = QueryParams::parse("name=hello%20world&q=a%2Bb").unwrap();

            assert_eq!(params.get("name"), Some("hello world"));
            assert_eq!(params.get("q"), Some("a+b"));
        }

        #[test]
        fn test_insert_and_set() {
            let mut params = QueryParams::new();

            params.insert("tags", "a");
            params.insert("tags", "b");
            assert_eq!(params.get_all("tags"), vec!["a", "b"]);

            params.set("tags", "c");
            assert_eq!(params.get_all("tags"), vec!["c"]);
        }

        #[test]
        fn test_get_as_types() {
            let params = QueryParams::parse("page=5&ratio=0.5&enabled=true").unwrap();

            assert_eq!(params.get_as::<i32>("page").unwrap(), 5);
            assert_eq!(params.get_as::<f64>("ratio").unwrap(), 0.5);
            assert_eq!(params.get_as::<bool>("enabled").unwrap(), true);
        }

        #[test]
        fn test_to_query_string() {
            let mut params = QueryParams::new();
            params.set("name", "john doe");
            params.set("age", "30");

            let query = params.to_query_string();
            assert!(query.contains("name=john+doe"));
            assert!(query.contains("age=30"));
        }

        #[test]
        fn test_remove() {
            let mut params = QueryParams::parse("a=1&b=2").unwrap();

            let removed = params.remove("a");
            assert_eq!(removed, Some(vec!["1".to_string()]));
            assert!(!params.contains("a"));
        }
    }

    mod parsed_path {
        use super::*;

        #[test]
        fn test_parse_root() {
            let path = ParsedPath::parse("/");
            assert_eq!(path.raw, "/");
            assert!(path.segments.is_empty());
            assert!(!path.trailing_slash);
        }

        #[test]
        fn test_parse_simple() {
            let path = ParsedPath::parse("/users/123");
            assert_eq!(path.segments, vec!["users", "123"]);
            assert!(!path.trailing_slash);
        }

        #[test]
        fn test_parse_trailing_slash() {
            let path = ParsedPath::parse("/users/");
            assert_eq!(path.segments, vec!["users"]);
            assert!(path.trailing_slash);
        }

        #[test]
        fn test_depth() {
            assert_eq!(ParsedPath::parse("/").depth(), 0);
            assert_eq!(ParsedPath::parse("/a").depth(), 1);
            assert_eq!(ParsedPath::parse("/a/b/c").depth(), 3);
        }

        #[test]
        fn test_parent() {
            let path = ParsedPath::parse("/users/123/profile");
            let parent = path.parent().unwrap();
            assert_eq!(parent.raw, "/users/123");

            let grandparent = parent.parent().unwrap();
            assert_eq!(grandparent.raw, "/users");

            let root = ParsedPath::parse("/").parent();
            assert!(root.is_none());
        }

        #[test]
        fn test_join() {
            let path = ParsedPath::parse("/users");

            let joined = path.join("123");
            assert_eq!(joined.raw, "/users/123");

            let relative = path.join("../posts");
            assert_eq!(relative.raw, "/posts");

            let absolute = ParsedPath::parse("/").join("a/b/c");
            assert_eq!(absolute.raw, "/a/b/c");
        }
    }

    mod parsed_url {
        use super::*;

        #[test]
        fn test_parse_path_only() {
            let url = ParsedUrl::parse("/users/123").unwrap();
            assert_eq!(url.path.raw, "/users/123");
            assert!(url.query.is_empty());
            assert!(url.fragment.is_none());
        }

        #[test]
        fn test_parse_with_query() {
            let url = ParsedUrl::parse("/search?q=rust&page=1").unwrap();
            assert_eq!(url.path.raw, "/search");
            assert_eq!(url.query.get("q"), Some("rust"));
            assert_eq!(url.query.get("page"), Some("1"));
        }

        #[test]
        fn test_parse_with_fragment() {
            let url = ParsedUrl::parse("/docs#section1").unwrap();
            assert_eq!(url.path.raw, "/docs");
            assert_eq!(url.fragment, Some("section1".to_string()));
        }

        #[test]
        fn test_parse_full_url() {
            let url = ParsedUrl::parse("/users/123?tab=posts#latest").unwrap();
            assert_eq!(url.path.raw, "/users/123");
            assert_eq!(url.query.get("tab"), Some("posts"));
            assert_eq!(url.fragment, Some("latest".to_string()));
        }

        #[test]
        fn test_to_string() {
            let url = ParsedUrl::parse("/users?page=1#top").unwrap();
            // Note: query string order may vary
            let reconstructed = url.to_string();
            assert!(reconstructed.starts_with("/users?"));
            assert!(reconstructed.contains("page=1"));
            assert!(reconstructed.ends_with("#top"));
        }
    }
}
