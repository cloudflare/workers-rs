use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsStringLike};

use crate::{
    Cors, Error, Headers, HeadersKind, ReadableStream, Response, ResponseInit, ResponseInitBuilder,
    Result, WebSocket,
};

impl Response {
    /// The generated `ResponseInit` builder, which also carries the `worker`-style
    /// terminal constructors (`ok`, `error`, `empty`, `from_json`, ...).
    pub fn builder() -> ResponseInitBuilder {
        ResponseInit::builder()
    }

    /// Splits the response into a builder holding its status, headers, `webSocket` and
    /// `cf`, plus the body stream, so a new response can be constructed with changes; a
    /// JS `Response` is immutable. `statusText` is not carried: the platform derives the
    /// default from the final status. `builder.body(body)` reassembles.
    pub fn into_parts(self) -> (ResponseInitBuilder, Option<ReadableStream<Uint8Array>>) {
        let mut builder = Response::builder()
            .status(self.status())
            .headers(&self.headers());
        if let Some(ws) = self.web_socket() {
            builder = builder.web_socket(&ws);
        }
        let cf = self.cf();
        if !cf.is_undefined() && !cf.is_null() {
            builder = builder.cf(&cf);
        }
        // Stopgap: `body` is annotated `ReadableStream<Uint8Array>`, which the generator does not read yet.
        (builder, self.body().map(JsCast::unchecked_into))
    }
}

/// `headers` may hold any `HeadersInit`; `new Headers(init)` normalizes all forms.
fn headers_of(init: &ResponseInit) -> Result<Headers> {
    Ok(match init.headers() {
        Some(HeadersKind::Headers(headers)) => Headers::new_with_headers(&headers)?,
        Some(HeadersKind::Record(record)) => Headers::new_with_record(&record)?,
        Some(HeadersKind::JsValue(pairs)) => Headers::new_with_headers(pairs.unchecked_ref())?,
        None => Headers::default(),
    })
}

// Stopgap for `From<ResponseInit> for ResponseInitBuilder`: resumes a chain from a built dictionary.
fn builder_from(init: ResponseInit) -> ResponseInitBuilder {
    let mut builder = ResponseInit::builder();
    if let Some(status) = init.status() {
        builder = builder.status(status);
    }
    if let Some(status_text) = init.status_text() {
        builder = builder.status_text(status_text);
    }
    match init.headers() {
        Some(HeadersKind::Headers(headers)) => builder = builder.headers(&headers),
        Some(HeadersKind::Record(record)) => builder = builder.headers_with_record(&record),
        Some(HeadersKind::JsValue(pairs)) => builder = builder.headers_with_iterable(&pairs),
        None => {}
    }
    let cf = init.cf();
    if !cf.is_undefined() {
        builder = builder.cf(&cf);
    }
    if let Some(ws) = init.web_socket() {
        builder = builder.web_socket(&ws);
    }
    if let Some(encode_body) = init.encode_body() {
        builder = builder.encode_body(encode_body);
    }
    builder
}

impl ResponseInitBuilder {
    /// Reads the headers set so far, normalized to a `Headers`, hands them to `edit`,
    /// and resumes the chain with the result stored back.
    fn edit_headers(self, edit: impl FnOnce(&Headers) -> Result<()>) -> Result<Self> {
        let init = self.build();
        let headers = headers_of(&init)?;
        edit(&headers)?;
        init.set_headers(&headers);
        Ok(builder_from(init))
    }

    /// Sets `content-type` unless one was already provided.
    fn with_content_type(self, content_type: &str) -> Result<ResponseInit> {
        Ok(self
            .edit_headers(|headers| {
                if !headers.has("content-type")? {
                    headers.set("content-type", content_type)?;
                }
                Ok(())
            })?
            .build())
    }

    /// Sets one header, keeping those set before.
    pub fn header(self, name: impl JsStringLike, value: impl JsStringLike) -> Result<Self> {
        self.edit_headers(|headers| Ok(headers.set(name, value)?))
    }

    /// Applies `cors` to the headers set so far.
    pub fn cors(self, cors: &Cors) -> Result<Self> {
        self.edit_headers(|headers| cors.apply_headers(headers))
    }

    /// A text body with `content-type: text/plain; charset=utf-8`.
    pub fn ok(self, body: impl JsStringLike) -> Result<Response> {
        let init = self.with_content_type("text/plain; charset=utf-8")?;
        Ok(Response::new_with_str_and_init(body, &init)?)
    }

    /// A text body with `content-type: text/html; charset=utf-8`.
    pub fn from_html(self, html: impl JsStringLike) -> Result<Response> {
        let init = self.with_content_type("text/html; charset=utf-8")?;
        Ok(Response::new_with_str_and_init(html, &init)?)
    }

    /// A body of raw bytes.
    pub fn from_bytes(self, bytes: impl AsRef<[u8]>) -> Result<Response> {
        // Stopgap: no bytes form of a buffer argument yet, so the copy goes through a `Uint8Array`.
        Ok(Response::new_with_typed_array_and_init(
            &Uint8Array::from(bytes.as_ref()),
            &self.build(),
        )?)
    }

    /// A response with no body. Required for null-body statuses such as 204 and 304.
    pub fn empty(self) -> Result<Response> {
        Ok(Response::new_with_readable_stream_and_init(
            None,
            &self.build(),
        )?)
    }

    /// A body taken from a JS stream, as returned by `Response::body()` or
    /// `into_parts()`; `None` is an empty body.
    pub fn body(self, body: Option<ReadableStream<Uint8Array>>) -> Result<Response> {
        match body {
            Some(stream) => Ok(Response::new_with_readable_stream_and_init(
                Some(&stream),
                &self.build(),
            )?),
            None => self.empty(),
        }
    }

    /// The `101 Switching Protocols` response that hands `ws` to the client.
    pub fn from_websocket(self, ws: WebSocket) -> Result<Response> {
        self.status(101).web_socket(&ws).empty()
    }

    /// A text body with an error status. `status` must be in `400..=599`.
    pub fn error(self, msg: impl JsStringLike, status: u16) -> Result<Response> {
        if !(400..=599).contains(&status) {
            return Err(Error::Rust(format!(
                "error status codes must be in the 400-599 range, got {status}"
            )));
        }
        Ok(Response::new_with_str_and_init(
            msg,
            &self.status(status).build(),
        )?)
    }
}
