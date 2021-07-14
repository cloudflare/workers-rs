use crate::{Error, Result};

use std::{
    iter::{FromIterator, Map},
    result::Result as StdResult,
    str::FromStr,
};

use edgeworker_sys::Headers as EdgeHeaders;
use http::{header::HeaderName, HeaderMap, HeaderValue};
use js_sys::Array;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct Headers(pub EdgeHeaders);

#[allow(clippy::new_without_default)]
impl Headers {
    pub fn new() -> Self {
        // This cannot throw an error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/Headers
        Headers(EdgeHeaders::new().unwrap())
    }

    // Returns an error if the name is invalid (e.g. contains spaces)
    pub fn get(&self, name: &str) -> Result<Option<String>> {
        self.0.get(name).map_err(Error::from)
    }

    // Returns an error if the name is invalid (e.g. contains spaces)
    pub fn has(&self, name: &str) -> Result<bool> {
        self.0.has(name).map_err(Error::from)
    }

    // Throws an error if the name is invalid (e.g. contains spaces)
    pub fn append(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.append(name, value).map_err(Error::from)
    }

    // Throws an error if the name is invalid (e.g. contains spaces)
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.0.set(name, value).map_err(Error::from)
    }

    // Throws an error if the name is invalid (e.g. contains spaces)
    // or if the JS Headers objects's guard is immutable (e.g. for an incoming request)
    pub fn delete(&mut self, name: &str) -> Result<()> {
        self.0.delete(name).map_err(Error::from)
    }

    pub fn entries(&self) -> HeaderIterator {
        self.0
            .entries()
            // Header.entries() doesn't error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/entries
            .unwrap()
            .into_iter()
            // The entries iterator.next() will always return a proper value: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols
            .map((|a| a.unwrap().into()) as F1)
            // The entries iterator always returns an array[2] of strings
            .map(|a: Array| (a.get(0).as_string().unwrap(), a.get(1).as_string().unwrap()))
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.0
            .keys()
            // Header.keys() doesn't error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/keys
            .unwrap()
            .into_iter()
            // The keys iterator.next() will always return a proper value containing a string
            .map(|a| a.unwrap().as_string().unwrap())
    }

    pub fn values(&self) -> impl Iterator<Item = String> {
        self.0
            .values()
            // Header.values() doesn't error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/values
            .unwrap()
            .into_iter()
            // The values iterator.next() will always return a proper value containing a string
            .map(|a| a.unwrap().as_string().unwrap())
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
        Headers(EdgeHeaders::new_with_headers(&self.0).unwrap())
    }
}
