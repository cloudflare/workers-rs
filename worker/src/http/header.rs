use crate::Result;
use http::{HeaderMap, HeaderName, HeaderValue};
use js_sys::Array;
use worker_sys::ext::HeadersExt;

pub(crate) fn header_map_from_web_sys_headers(
    from_headers: web_sys::Headers,
    to_headers: &mut HeaderMap,
) -> Result<()> {
    // Header.entries() doesn't error: https://developer.mozilla.org/en-US/docs/Web/API/Headers/entries
    for res in from_headers.entries()?.into_iter() {
        // The entries iterator.next() will always return a proper value: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols
        let a: Array = res?.into();
        // The entries iterator always returns an array[2] of strings
        let key = a.get(0).as_string().unwrap();
        let value = a.get(1).as_string().unwrap();
        to_headers.append(
            HeaderName::from_bytes(key.as_bytes())?,
            HeaderValue::from_str(&value)?,
        );
    }
    Ok(())
}

pub(crate) fn web_sys_headers_from_header_map(headers: &HeaderMap) -> Result<web_sys::Headers> {
    let output = web_sys::Headers::new()?;
    for (key, value) in headers.iter() {
        output.append(key.as_str(), std::str::from_utf8(value.as_bytes())?)?;
    }
    Ok(output)
}
