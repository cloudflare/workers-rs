use std::marker::PhantomData;

use edgeworker_ffi::headers::Headers as FfiHeaders;
use js_sys::{IteratorNext, JsString};

use crate::error::WorkerError;

/// All HTTP request and response headers are available through the [Headers API](https://developer.mozilla.org/en-US/docs/Web/API/Headers).
///
/// [Read more](https://developers.cloudflare.com/workers/runtime-apis/headers)
#[derive(Debug, Clone)]
pub struct Headers {
    inner: FfiHeaders,
}

/// An HTTP header
#[derive(Debug, Clone)]
pub struct Header {
    /// The name of the header (e.g. `Content-Type`)
    pub name: String,
    /// The value of the header (e.g. `text/html`)
    pub value: String,
}

impl Headers {
    /// Creates an empty Headers.
    ///
    /// To initialize with values, use `Headers::with_values`.
    pub fn new() -> Self {
        Self {
            // safety: we're not using any outlandish parameters, so this is fine.
            inner: FfiHeaders::new().unwrap(),
        }
    }

    /// Create a Headers with some values pre-filled.
    pub fn with_values<I>(collection: I) -> Self
    where
        I: IntoIterator<Item = Header>,
    {
        let headers = Headers::new();
        collection
            .into_iter()
            .for_each(|Header { name, value }| headers.append(name, value));
        headers
    }

    /// The append() method of the Headers interface appends a new value onto an existing header inside a Headers object, or adds the header if it does not already exist.
    ///
    /// The difference between set() and append() is that if the specified header already exists and accepts multiple values, set() will overwrite the existing value with the new one, whereas append() will append the new value onto the end of the set of values.
    ///
    /// When a header name possesses multiple values, those values will be concatenated as a single, comma-delimited string value.
    pub fn append<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        // I don't even really understand how this can panic
        self.inner.append(name.as_ref(), value.as_ref()).unwrap();
    }

    /// Deletes a header from a Headers object.
    pub fn delete<K: AsRef<str>>(&self, name: K) -> Result<(), WorkerError> {
        self.inner
            .delete(name.as_ref())
            .map_err(|e| WorkerError::JsError {
                message: format!("Failed to delete header {}", name.as_ref()),
                js_error: e,
            })
    }

    /// Returns a String of all the values of a header within a Headers object with a given name.
    pub fn get<K: AsRef<str>>(&self, name: K) -> Option<String> {
        // safety: i think this is fine, because...the error would be if
        // it didn't exist...so i'm not sure the underlying binding is
        // right, actually. but
        self.inner.get(name.as_ref()).unwrap()
    }

    /// Returns a `bool` stating whether a Headers object contains a certain header.
    pub fn has<K: AsRef<str>>(&self, name: K) -> bool {
        self.inner.has(name.as_ref()).unwrap_or(false)
    }

    /// Sets a new value for an existing header inside a Headers object, or adds the header if it does not already exist.
    ///
    /// This will completely replace any old header value, unlike `append`, which will add it to the existing value
    pub fn set<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) -> Result<(), WorkerError> {
        self.inner
            .set(name.as_ref(), value.as_ref())
            .map_err(|e| WorkerError::JsError {
                message: format!(
                    "Failed to set header {} to {}",
                    name.as_ref(),
                    value.as_ref()
                ),
                js_error: e,
            })
    }

    /// Returns an iterator over the headers as key/value pairs
    pub fn entries(&self) -> impl Iterator<Item = Header> {
        // safety: i think...this should always be safe...right?
        HeadersIterator::<Tuple>::new(self.inner.entries().unwrap())
    }

    /// Returns an iterator over the names of the headers (e.g. just the keys)
    pub fn keys(&self) -> impl Iterator<Item = String> {
        // safety: i think...this should always be safe...right?
        HeadersIterator::<Singleton>::new(self.inner.keys().unwrap())
    }

    /// Returns an iterator over the values of the headers (e.g. just the values)
    pub fn values(&self) -> impl Iterator<Item = String> {
        // safety: i think...this should always be safe...right?
        HeadersIterator::<Singleton>::new(self.inner.values().unwrap())
    }
}

impl From<FfiHeaders> for Headers {
    fn from(inner: FfiHeaders) -> Self {
        Self { inner }
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

struct HeadersIterator<T> {
    inner: js_sys::Iterator,
    iter_values: PhantomData<T>,
}

impl<T> HeadersIterator<T> {
    pub fn new(inner: js_sys::Iterator) -> Self {
        Self {
            inner,
            iter_values: PhantomData,
        }
    }
}

struct Singleton;
struct Tuple;

impl Iterator for HeadersIterator<Singleton> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // unwrap because we trust that the inner iterator is an iterator
        let iter_result: IteratorNext = self.inner.next().unwrap();

        if iter_result.done() {
            None
        } else {
            let parsed = JsString::from(iter_result.value()).into();
            Some(parsed)
        }
    }
}

impl Iterator for HeadersIterator<Tuple> {
    type Item = Header;

    fn next(&mut self) -> Option<Self::Item> {
        let iter_result: IteratorNext = self.inner.next().unwrap();

        if iter_result.done() {
            None
        } else {
            // https://developer.mozilla.org/en-US/docs/Web/API/Headers/entries#example
            // they're length 2 arrays
            let arr = js_sys::Array::from(&iter_result.value());

            let name = JsString::from(arr.get(0)).into();
            let value = JsString::from(arr.get(0)).into();

            Some(Header { name, value })
        }
    }
}
