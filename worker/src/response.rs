use crate::cors::Cors;
use crate::error::Error;
use crate::headers::Headers;
use crate::ByteStream;
use crate::Result;
use crate::WebSocket;

#[cfg(feature = "http")]
use bytes::Bytes;
use futures_util::{TryStream, TryStreamExt};
use js_sys::Uint8Array;
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "http")]
use std::convert::TryFrom;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::ReadableStream;
use worker_sys::ext::{ResponseExt, ResponseInitExt};

#[derive(Debug, Clone)]
pub enum ResponseBody {
    Empty,
    Body(Vec<u8>),
    Stream(ReadableStream),
}

const CONTENT_TYPE: &str = "content-type";

/// A [Response](https://developer.mozilla.org/en-US/docs/Web/API/Response) representation for
/// working with or returning a response to a `Request`.
#[derive(Debug)]
pub struct Response {
    body: ResponseBody,
    headers: Headers,
    status_code: u16,
    websocket: Option<WebSocket>,
}

#[cfg(feature = "http")]
impl<B: http_body::Body<Data = Bytes> + 'static> TryFrom<http::Response<B>> for Response {
    type Error = crate::Error;
    fn try_from(res: http::Response<B>) -> Result<Self> {
        let resp = crate::http::response::to_wasm(res)?;
        Ok(resp.into())
    }
}

#[cfg(feature = "http")]
impl TryFrom<Response> for crate::HttpResponse {
    type Error = crate::Error;
    fn try_from(res: Response) -> Result<crate::HttpResponse> {
        let sys_resp: web_sys::Response = res.into();
        crate::http::response::from_wasm(sys_resp)
    }
}

impl Response {
    /// Create a `Response` using `B` as the body encoded as JSON. Sets the associated
    /// `Content-Type` header for the `Response` as `application/json`.
    pub fn from_json<B: Serialize>(value: &B) -> Result<Self> {
        if let Ok(data) = serde_json::to_string(value) {
            let mut headers = Headers::new();
            headers.set(CONTENT_TYPE, "application/json")?;

            return Ok(Self {
                body: ResponseBody::Body(data.into_bytes()),
                headers,
                status_code: 200,
                websocket: None,
            });
        }

        Err(Error::Json(("Failed to encode data to json".into(), 500)))
    }

    /// Create a `Response` using the body encoded as HTML. Sets the associated `Content-Type`
    /// header for the `Response` as `text/html; charset=utf-8`.
    pub fn from_html(html: impl AsRef<str>) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set(CONTENT_TYPE, "text/html; charset=utf-8")?;

        let data = html.as_ref().as_bytes().to_vec();
        Ok(Self {
            body: ResponseBody::Body(data),
            headers,
            status_code: 200,
            websocket: None,
        })
    }

    /// Create a `Response` using unprocessed bytes provided. Sets the associated `Content-Type`
    /// header for the `Response` as `application/octet-stream`.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set(CONTENT_TYPE, "application/octet-stream")?;

        Ok(Self {
            body: ResponseBody::Body(bytes),
            headers,
            status_code: 200,
            websocket: None,
        })
    }

    /// Create a `Response` using a `ResponseBody` variant. Sets a status code of 200 and an empty
    /// set of Headers. Modify the Response with methods such as `with_status` and `with_headers`.
    pub fn from_body(body: ResponseBody) -> Result<Self> {
        Ok(Self {
            body,
            headers: Headers::new(),
            status_code: 200,
            websocket: None,
        })
    }

    /// Create a `Response` using a `WebSocket` client. Configures the browser to switch protocols
    /// (using status code 101) and returns the websocket.
    pub fn from_websocket(websocket: WebSocket) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            headers: Headers::new(),
            status_code: 101,
            websocket: Some(websocket),
        })
    }

    /// Create a `Response` using a [`Stream`](futures::stream::Stream) for the body. Sets a status
    /// code of 200 and an empty set of Headers. Modify the Response with methods such as
    /// `with_status` and `with_headers`.
    pub fn from_stream<S>(stream: S) -> Result<Self>
    where
        S: TryStream + 'static,
        S::Ok: Into<Vec<u8>>,
        S::Error: Into<Error>,
    {
        let js_stream = stream
            .map_ok(|item| -> Vec<u8> { item.into() })
            .map_ok(|chunk| {
                let array = Uint8Array::new_with_length(chunk.len() as _);
                array.copy_from(&chunk);

                array.into()
            })
            .map_err(|err| -> crate::Error { err.into() })
            .map_err(|e| JsValue::from(e.to_string()));

        let stream = wasm_streams::ReadableStream::from_stream(js_stream);
        let stream: ReadableStream = stream.into_raw().dyn_into().unwrap();

        let edge_res = web_sys::Response::new_with_opt_readable_stream(Some(&stream))?;
        Ok(Self::from(edge_res))
    }

    /// Create a `Response` using unprocessed text provided. Sets the associated `Content-Type`
    /// header for the `Response` as `text/plain; charset=utf-8`.
    pub fn ok(body: impl Into<String>) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set(CONTENT_TYPE, "text/plain; charset=utf-8")?;

        Ok(Self {
            body: ResponseBody::Body(body.into().into_bytes()),
            headers,
            status_code: 200,
            websocket: None,
        })
    }

    /// Create an empty `Response` with a 200 status code.
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            headers: Headers::new(),
            status_code: 200,
            websocket: None,
        })
    }

    /// A helper method to send an error message to a client. Will return `Err` if the status code
    /// provided is outside the valid HTTP error range of 400-599.
    pub fn error(msg: impl Into<String>, status: u16) -> Result<Self> {
        if !(400..=599).contains(&status) {
            return Err(Error::Internal(
                "error status codes must be in the 400-599 range! see https://developer.mozilla.org/en-US/docs/Web/HTTP/Status for more".into(),
            ));
        }

        Ok(Self {
            body: ResponseBody::Body(msg.into().into_bytes()),
            headers: Headers::new(),
            status_code: status,
            websocket: None,
        })
    }

    /// Create a `Response` which redirects to the specified URL with default status_code of 302
    pub fn redirect(url: url::Url) -> Result<Self> {
        match web_sys::Response::redirect(url.as_str()) {
            Ok(edge_response) => Ok(Response::from(edge_response)),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Create a `Response` which redirects to the specified URL with a custom status_code
    pub fn redirect_with_status(url: url::Url, status_code: u16) -> Result<Self> {
        if !(300..=399).contains(&status_code) {
            return Err(Error::Internal(
                "redirect status codes must be in the 300-399 range! Please checkout https://developer.mozilla.org/en-US/docs/Web/HTTP/Status#redirection_messages for more".into(),
            ));
        }
        match web_sys::Response::redirect_with_status(url.as_str(), status_code) {
            Ok(edge_response) => Ok(Response::from(edge_response)),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Get the HTTP Status code of this `Response`.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Access this response's body
    pub fn body(&self) -> &ResponseBody {
        &self.body
    }

    /// Access this response's body as plaintext.
    pub async fn text(&mut self) -> Result<String> {
        match &self.body {
            ResponseBody::Body(bytes) => {
                Ok(String::from_utf8(bytes.clone()).map_err(|e| Error::from(e.to_string()))?)
            }
            ResponseBody::Empty => Ok(String::new()),
            ResponseBody::Stream(_) => {
                let bytes = self.bytes().await?;
                String::from_utf8(bytes).map_err(|e| Error::RustError(e.to_string()))
            }
        }
    }

    /// Access this response's body encoded as JSON.
    pub async fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
        let content_type = self.headers().get(CONTENT_TYPE)?.unwrap_or_default();
        if !content_type.contains("application/json") {
            return Err(Error::BadEncoding);
        }
        serde_json::from_str(&self.text().await?).map_err(Error::from)
    }

    /// Access this response's body encoded as raw bytes.
    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(bytes.clone()),
            ResponseBody::Empty => Ok(Vec::new()),
            ResponseBody::Stream(_) => {
                self.stream()?
                    .try_fold(Vec::new(), |mut bytes, mut chunk| async move {
                        bytes.append(&mut chunk);
                        Ok(bytes)
                    })
                    .await
            }
        }
    }

    /// Access this response's body as a [`Stream`](futures::stream::Stream) of bytes.
    pub fn stream(&mut self) -> Result<ByteStream> {
        let stream = match &self.body {
            ResponseBody::Stream(edge_request) => edge_request.clone(),
            _ => return Err(Error::RustError("body is not streamable".into())),
        };

        let stream = wasm_streams::ReadableStream::from_raw(stream.dyn_into().unwrap());

        Ok(ByteStream {
            inner: stream.into_stream(),
        })
    }

    // Get the WebSocket returned by the the server.
    pub fn websocket(self) -> Option<WebSocket> {
        self.websocket
    }

    /// Set this response's `Headers`.
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Set this response's status code.
    /// The Workers platform will reject HTTP status codes outside the range of 200..599 inclusive,
    /// and will throw a JavaScript `RangeError`, returning a response with an HTTP 500 status code.
    pub fn with_status(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    /// Sets this response's cors headers from the `Cors` struct.
    /// Example usage:
    /// ```
    /// use worker::*;
    /// fn fetch() -> worker::Result<Response> {
    ///     let cors = Cors::default();
    ///     Response::empty()?
    ///         .with_cors(&cors)
    /// }
    /// ```
    pub fn with_cors(self, cors: &Cors) -> Result<Self> {
        let mut headers = self.headers.clone();
        cors.apply_headers(&mut headers)?;
        Ok(self.with_headers(headers))
    }

    /// Sets this response's `webSocket` option.
    /// This will require a status code 101 to work.
    pub fn with_websocket(mut self, websocket: Option<WebSocket>) -> Self {
        self.websocket = websocket;
        self
    }

    /// Read the `Headers` on this response.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` on this response.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Clones the response so it can be used multiple times.
    pub fn cloned(&mut self) -> Result<Self> {
        if self.websocket.is_some() {
            return Err(Error::RustError("WebSockets cannot be cloned".into()));
        }

        let edge = web_sys::Response::from(&*self);
        let cloned = edge.clone()?;

        // Cloning a response might modify it's body as it might need to tee the stream, so we'll
        // need to update it.
        self.body = match edge.body() {
            Some(stream) => ResponseBody::Stream(stream),
            None => ResponseBody::Empty,
        };

        Ok(cloned.into())
    }
}

#[test]
fn no_using_invalid_error_status_code() {
    assert!(Response::error("OK", 200).is_err());
    assert!(Response::error("600", 600).is_err());
    assert!(Response::error("399", 399).is_err());
}

pub struct ResponseInit {
    pub status: u16,
    pub headers: Headers,
    pub websocket: Option<WebSocket>,
}

impl From<ResponseInit> for web_sys::ResponseInit {
    fn from(init: ResponseInit) -> Self {
        let mut edge_init = web_sys::ResponseInit::new();
        edge_init.status(init.status);
        edge_init.headers(&init.headers.0);
        if let Some(websocket) = &init.websocket {
            edge_init.websocket(websocket.as_ref());
        }
        edge_init
    }
}

impl From<Response> for web_sys::Response {
    fn from(res: Response) -> Self {
        match res.body {
            ResponseBody::Body(bytes) => {
                let array = Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(&bytes);
                web_sys::Response::new_with_opt_buffer_source_and_init(
                    Some(&array),
                    &ResponseInit {
                        status: res.status_code,
                        headers: res.headers,
                        websocket: res.websocket,
                    }
                    .into(),
                )
                .unwrap()
            }
            ResponseBody::Stream(stream) => {
                web_sys::Response::new_with_opt_readable_stream_and_init(
                    Some(&stream),
                    &ResponseInit {
                        status: res.status_code,
                        headers: res.headers,
                        websocket: res.websocket,
                    }
                    .into(),
                )
                .unwrap()
            }
            ResponseBody::Empty => web_sys::Response::new_with_opt_str_and_init(
                None,
                &ResponseInit {
                    status: res.status_code,
                    headers: res.headers,
                    websocket: res.websocket,
                }
                .into(),
            )
            .unwrap(),
        }
    }
}

impl From<&Response> for web_sys::Response {
    fn from(res: &Response) -> Self {
        match &res.body {
            ResponseBody::Body(bytes) => {
                let array = Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(bytes);
                web_sys::Response::new_with_opt_buffer_source_and_init(
                    Some(&array),
                    &ResponseInit {
                        status: res.status_code,
                        headers: res.headers.clone(),
                        websocket: res.websocket.clone(),
                    }
                    .into(),
                )
                .unwrap()
            }
            ResponseBody::Stream(stream) => {
                web_sys::Response::new_with_opt_readable_stream_and_init(
                    Some(stream),
                    &ResponseInit {
                        status: res.status_code,
                        headers: res.headers.clone(),
                        websocket: res.websocket.clone(),
                    }
                    .into(),
                )
                .unwrap()
            }
            ResponseBody::Empty => web_sys::Response::new_with_opt_str_and_init(
                None,
                &ResponseInit {
                    status: res.status_code,
                    headers: res.headers.clone(),
                    websocket: res.websocket.clone(),
                }
                .into(),
            )
            .unwrap(),
        }
    }
}

impl From<web_sys::Response> for Response {
    fn from(res: web_sys::Response) -> Self {
        Self {
            headers: Headers(res.headers()),
            status_code: res.status(),
            websocket: res.websocket().map(|ws| ws.into()),
            body: match res.body() {
                Some(stream) => ResponseBody::Stream(stream),
                None => ResponseBody::Empty,
            },
        }
    }
}
