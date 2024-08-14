use crate::{error::Error, Result};

use std::{
    iter::{FromIterator, Map},
    result::Result as StdResult,
    str::FromStr,
};

use http::{header::HeaderName, HeaderMap, HeaderValue};
use js_sys::Array;
use wasm_bindgen::JsValue;
use worker_sys::ext::HeadersExt;

/// A [Headers](https://developer.mozilla.org/en-US/docs/Web/API/Headers) representation used in
/// Request and Response objects.
pub struct Headers(pub web_sys::Headers);

impl std::fmt::Debug for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Headers {\n")?;
        for (k, v) in self.entries() {
            f.write_str(&format!("{k} = {v}\n"))?;
        }
        f.write_str("}\n")
    }
}

impl Headers {
    /// Construct a new `Headers` struct.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns all the values of a header within a `Headers` object with a given name.
    /// Returns an error if the name is invalid (e.g. contains spaces)
    pub fn get(&self, name: &str) -> Result<Option<String>> {
        self.0.get(name).map_err(Error::from)
    }

    /// Returns a boolean stating whether a `Headers` object contains a certain header.
    /// Returns an error if the name is invalid (e.g. contains spaces)
    pub fn has(&self, name: &str) -> Result<bool> {
        self.0.has(name).map_err(Error::from)
    }

    /// Returns an error if the name is invalid (e.g. contains spaces)
    pub fn append(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.append(name, value).map_err(Error::from)
    }

    /// Sets a new value for an existing header inside a `Headers` object, or adds the header if it does not already exist.
    /// Returns an error if the name is invalid (e.g. contains spaces)
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.set(name, value).map_err(Error::from)
    }

    /// Deletes a header from a `Headers` object.
    /// Returns an error if the name is invalid (e.g. contains spaces)
    /// or if the JS Headers object's guard is immutable (e.g. for an incoming request)
    pub fn delete(&mut self, name: &str) -> Result<()> {
        self.0.delete(name).map_err(Error::from)
    }

    /// Returns an iterator allowing to go through all key/value pairs contained in this object.
    pub fn entries(&self) -> HeaderIterator {
        self.0
            .entries()
            .into_iter()
            // The entries iterator.next() will always return a proper value: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols
            .map((|a| a.unwrap().into()) as F1)
            // The entries iterator always returns an array[2] of strings
            .map(|a: Array| (a.get(0).as_string().unwrap(), a.get(1).as_string().unwrap()))
    }

    /// Returns an iterator allowing you to go through all keys of the key/value pairs contained in
    /// this object.
    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.0
            .keys()
            .into_iter()
            // The keys iterator.next() will always return a proper value containing a string
            .map(|a| a.unwrap().as_string().unwrap())
    }

    /// Returns an iterator allowing you to go through all values of the key/value pairs contained
    /// in this object.
    pub fn values(&self) -> impl Iterator<Item = String> {
        self.0
            .values()
            .into_iter()
            // The values iterator.next() will always return a proper value containing a string
            .map(|a| a.unwrap().as_string().unwrap())
    }
}

impl Default for Headers {
    fn default() -> Self {
        // This cannot throw an error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/Headers
        Headers(web_sys::Headers::new().unwrap())
    }
}

type F1 = fn(StdResult<JsValue, JsValue>) -> Array;
type HeaderIterator = Map<Map<js_sys::IntoIter, F1>, fn(Array) -> (String, String)>;

impl IntoIterator for &Headers {
    type Item = (String, String);

    type IntoIter = HeaderIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.entries()
    }
}

impl<T: AsRef<str>> FromIterator<(T, T)> for Headers {
    fn from_iter<U: IntoIterator<Item = (T, T)>>(iter: U) -> Self {
        let mut headers = Headers::new();
        iter.into_iter().for_each(|(name, value)| {
            headers.append(name.as_ref(), value.as_ref()).ok();
        });
        headers
    }
}

impl<'a, T: AsRef<str>> FromIterator<&'a (T, T)> for Headers {
    fn from_iter<U: IntoIterator<Item = &'a (T, T)>>(iter: U) -> Self {
        let mut headers = Headers::new();
        iter.into_iter().for_each(|(name, value)| {
            headers.append(name.as_ref(), value.as_ref()).ok();
        });
        headers
    }
}

impl AsRef<JsValue> for Headers {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl From<&HeaderMap> for Headers {
    fn from(map: &HeaderMap) -> Self {
        map.keys()
            .flat_map(|name| {
                map.get_all(name)
                    .into_iter()
                    .map(move |value| (name.to_string(), value.to_str().unwrap().to_owned()))
            })
            .collect()
    }
}

impl From<HeaderMap> for Headers {
    fn from(map: HeaderMap) -> Self {
        (&map).into()
    }
}

impl From<&Headers> for HeaderMap {
    fn from(headers: &Headers) -> Self {
        headers
            .into_iter()
            .map(|(name, value)| {
                (
                    HeaderName::from_str(&name).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                )
            })
            .collect()
    }
}

impl From<Headers> for HeaderMap {
    fn from(headers: Headers) -> Self {
        (&headers).into()
    }
}

impl Clone for Headers {
    fn clone(&self) -> Self {
        // Headers constructor doesn't throw an error
        Headers(web_sys::Headers::new_with_headers(&self.0).unwrap())
    }
}
