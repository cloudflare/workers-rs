use crate::Headers;

impl Default for Headers {
    fn default() -> Self {
        // `new Headers()` cannot throw.
        Headers::new().unwrap()
    }
}

impl IntoIterator for &Headers {
    type Item = (String, String);
    type IntoIter = HeadersIter;

    fn into_iter(self) -> Self::IntoIter {
        HeadersIter(self.entries().into_iter())
    }
}

pub struct HeadersIter(js_sys::IntoIter<js_sys::ArrayTuple<(js_sys::JsString, js_sys::JsString)>>);

impl Iterator for HeadersIter {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let (name, value) = self.0.next()?.ok()?.into_tuple();
        Some((name.into(), value.into()))
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

    use crate::Headers;

    impl From<&HeaderMap> for Headers {
        fn from(map: &HeaderMap) -> Self {
            map.iter()
                .filter_map(|(name, value)| Some((name.as_str(), value.to_str().ok()?)))
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
                .filter_map(|(name, value)| {
                    Some((
                        HeaderName::from_str(&name).ok()?,
                        HeaderValue::from_str(&value).ok()?,
                    ))
                })
                .collect()
        }
    }

    impl From<Headers> for HeaderMap {
        fn from(headers: Headers) -> Self {
            (&headers).into()
        }
    }
}
