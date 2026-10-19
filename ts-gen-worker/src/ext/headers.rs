use crate::Headers;

impl Default for Headers {
    fn default() -> Self {
        // `new Headers()` cannot throw.
        Headers::new().unwrap()
    }
}

impl<K: AsRef<str>, V: AsRef<str>> FromIterator<(K, V)> for Headers {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let headers = Headers::default();
        for (name, value) in iter {
            // `collect` has no error channel: an invalid header rethrows the TypeError.
            if let Err(err) = headers.append(name.as_ref(), value.as_ref()) {
                wasm_bindgen::throw_val(err.into());
            }
        }
        headers
    }
}

impl<'a, K: AsRef<str>, V: AsRef<str>> FromIterator<&'a (K, V)> for Headers {
    fn from_iter<I: IntoIterator<Item = &'a (K, V)>>(iter: I) -> Self {
        iter.into_iter()
            .map(|(name, value)| (name.as_ref(), value.as_ref()))
            .collect()
    }
}

#[cfg(feature = "http")]
mod http_impls {
    use std::str::FromStr;

    use http::{HeaderMap, HeaderName, HeaderValue};

    use crate::{Error, Headers, Result};

    /// A header `http` holds as bytes but JS cannot (a non-ASCII value), or a name or
    /// value the platform rejects, is an error rather than a silently dropped entry.
    impl TryFrom<&HeaderMap> for Headers {
        type Error = Error;

        fn try_from(map: &HeaderMap) -> Result<Self> {
            let headers = Headers::default();
            for (name, value) in map {
                let value = value.to_str().map_err(|e| {
                    Error::Rust(format!("header `{name}` has a non-ASCII value: {e}"))
                })?;
                headers.append(name.as_str(), value)?;
            }
            Ok(headers)
        }
    }

    impl TryFrom<HeaderMap> for Headers {
        type Error = Error;

        fn try_from(map: HeaderMap) -> Result<Self> {
            (&map).try_into()
        }
    }

    impl TryFrom<&Headers> for HeaderMap {
        type Error = Error;

        fn try_from(headers: &Headers) -> Result<Self> {
            let mut map = HeaderMap::new();
            for (name, value) in headers {
                let name = HeaderName::from_str(&name)
                    .map_err(|e| Error::Rust(format!("invalid header name `{name}`: {e}")))?;
                let value = HeaderValue::from_str(&value).map_err(|e| {
                    Error::Rust(format!("header `{name}` has an invalid value: {e}"))
                })?;
                map.append(name, value);
            }
            Ok(map)
        }
    }

    impl TryFrom<Headers> for HeaderMap {
        type Error = Error;

        fn try_from(headers: Headers) -> Result<Self> {
            (&headers).try_into()
        }
    }
}
