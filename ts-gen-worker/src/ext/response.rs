use js_sys::Uint8Array;
use wasm_bindgen::JsCast;

use crate::{Error, Headers, Response, ResponseInit, ResponseInitBuilder, Result};

impl Response {
    /// The generated `ResponseInit` builder, which also carries the `worker`-style
    /// terminal constructors (`ok`, `error`, `empty`, `from_json`, ...).
    pub fn builder() -> ResponseInitBuilder {
        ResponseInit::builder()
    }
}

/// `headers` may hold any `HeadersInit`; `new Headers(init)` normalizes all forms.
fn headers_of(init: &ResponseInit) -> Result<Headers> {
    let value = init.headers();
    Ok(if value.is_undefined() || value.is_null() {
        Headers::default()
    } else {
        Headers::new_with_headers(value.unchecked_ref())?
    })
}

impl ResponseInitBuilder {
    /// Sets `content-type` unless one was already provided.
    fn with_content_type(self, content_type: &str) -> Result<ResponseInit> {
        let init = self.build();
        let headers = headers_of(&init)?;
        if !headers.has("content-type")? {
            headers.set("content-type", content_type)?;
        }
        init.set_headers(&headers);
        Ok(init)
    }

    /// A text body with `content-type: text/plain; charset=utf-8`.
    pub fn ok(self, body: impl AsRef<str>) -> Result<Response> {
        let init = self.with_content_type("text/plain; charset=utf-8")?;
        Ok(Response::new_with_str_and_init(body.as_ref(), &init)?)
    }

    /// A text body with `content-type: text/html; charset=utf-8`.
    pub fn from_html(self, html: impl AsRef<str>) -> Result<Response> {
        let init = self.with_content_type("text/html; charset=utf-8")?;
        Ok(Response::new_with_str_and_init(html.as_ref(), &init)?)
    }

    /// A body of raw bytes.
    pub fn from_bytes(self, bytes: impl AsRef<[u8]>) -> Result<Response> {
        let buffer = Uint8Array::from(bytes.as_ref()).buffer();
        Ok(Response::new_with_array_buffer_and_init(
            &buffer,
            &self.build(),
        )?)
    }

    /// A response with no body. Required for null-body statuses such as 204 and 304.
    pub fn empty(self) -> Result<Response> {
        Ok(Response::new_with_null_and_init(
            &js_sys::Null::NULL,
            &self.build(),
        )?)
    }

    /// A text body with an error status. `status` must be in `400..=599`.
    pub fn error(self, msg: impl AsRef<str>, status: u16) -> Result<Response> {
        if !(400..=599).contains(&status) {
            return Err(Error::Rust(format!(
                "error status codes must be in the 400-599 range, got {status}"
            )));
        }
        Ok(Response::new_with_str_and_init(
            msg.as_ref(),
            &self.status(status).build(),
        )?)
    }
}
