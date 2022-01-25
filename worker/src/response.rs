use crate::error::Error;
use crate::headers::Headers;
use crate::Result;

use js_sys::Uint8Array;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen_futures::JsFuture;
use worker_sys::{Response as EdgeResponse, ResponseInit as EdgeResponseInit};

#[derive(Debug)]
pub enum ResponseBody {
    Empty,
    Body(Vec<u8>),
    Stream(EdgeResponse),
}

const CONTENT_TYPE: &str = "content-type";

/// A [Response](https://developer.mozilla.org/en-US/docs/Web/API/Response) representation for
/// working with or returning a response to a `Request`.
#[derive(Debug)]
pub struct Response {
    body: ResponseBody,
    headers: Headers,
    status_code: u16,
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
            });
        }

        Err(Error::Json(("Failed to encode data to json".into(), 500)))
    }

    /// Create a `Response` using the body encoded as HTML. Sets the associated `Content-Type`
    /// header for the `Response` as `text/html`.
    pub fn from_html(html: impl AsRef<str>) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set(CONTENT_TYPE, "text/html")?;

        let data = html.as_ref().as_bytes().to_vec();
        Ok(Self {
            body: ResponseBody::Body(data),
            headers,
            status_code: 200,
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
        })
    }

    // Create a `Response` using a `ResponseBody` variant. Sets a status code of 200 and an empty
    // set of Headers. Modify the Response with methods such as `with_status` and `with_headers`.
    pub fn from_body(body: ResponseBody) -> Result<Self> {
        Ok(Self {
            body,
            headers: Headers::new(),
            status_code: 200,
        })
    }

    /// Create a `Response` using unprocessed text provided. Sets the associated `Content-Type`
    /// header for the `Response` as `text/plain`.
    pub fn ok(body: impl Into<String>) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set(CONTENT_TYPE, "text/plain")?;

        Ok(Self {
            body: ResponseBody::Body(body.into().into_bytes()),
            headers,
            status_code: 200,
        })
    }

    /// Create an empty `Response` with a 200 status code.
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            headers: Headers::new(),
            status_code: 200,
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
        })
    }

    /// Create a `Response` which redirects to the specified URL with default status_code of 302
    pub fn redirect(url: url::Url) -> Result<Self> {
        match EdgeResponse::redirect(url.as_str()) {
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
        match EdgeResponse::redirect_with_status(url.as_str(), status_code) {
            Ok(edge_response) => Ok(Response::from(edge_response)),
            Err(err) => Err(Error::from(err)),
        }
    }

    /// Get the HTTP Status code of this `Response`.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Access this response's body as plaintext.
    pub async fn text(&mut self) -> Result<String> {
        match &self.body {
            ResponseBody::Body(bytes) => {
                Ok(String::from_utf8(bytes.clone()).map_err(|e| Error::from(e.to_string()))?)
            }
            ResponseBody::Empty => Ok(String::new()),
            ResponseBody::Stream(response) => JsFuture::from(response.text()?)
                .await
                .map(|value| value.as_string().unwrap())
                .map_err(Error::from),
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
            ResponseBody::Stream(response) => JsFuture::from(response.array_buffer()?)
                .await
                .map(|value| js_sys::Uint8Array::new(&value).to_vec())
                .map_err(Error::from),
        }
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

    /// Read the `Headers` on this response.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` on this response.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
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
}

impl From<ResponseInit> for EdgeResponseInit {
    fn from(init: ResponseInit) -> Self {
        let mut edge_init = EdgeResponseInit::new();
        edge_init.status(init.status);
        edge_init.headers(&init.headers.0);
        edge_init
    }
}

impl From<Response> for EdgeResponse {
    fn from(res: Response) -> Self {
        match res.body {
            ResponseBody::Body(bytes) => {
                let array = Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(&bytes);

                EdgeResponse::new_with_opt_u8_array_and_init(
                    Some(array),
                    &ResponseInit {
                        status: res.status_code,
                        headers: res.headers,
                    }
                    .into(),
                )
                .unwrap()
            }
            ResponseBody::Stream(response) => EdgeResponse::new_with_opt_stream_and_init(
                response.body(),
                &ResponseInit {
                    status: res.status_code,
                    headers: res.headers,
                }
                .into(),
            )
            .unwrap(),
            ResponseBody::Empty => EdgeResponse::new_with_opt_str_and_init(
                None,
                &ResponseInit {
                    status: res.status_code,
                    headers: res.headers,
                }
                .into(),
            )
            .unwrap(),
        }
    }
}

impl From<EdgeResponse> for Response {
    fn from(res: EdgeResponse) -> Self {
        Self {
            headers: Headers(res.headers()),
            status_code: res.status(),
            body: match res.body() {
                Some(_) => ResponseBody::Stream(res),
                None => ResponseBody::Empty,
            },
        }
    }
}
