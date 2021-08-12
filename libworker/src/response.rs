use crate::error::Error;
use crate::headers::Headers;
use crate::Result;

use edgeworker_ffi::{Response as EdgeResponse, ResponseInit as EdgeResponseInit};
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug)]
pub enum ResponseBody {
    Empty,
    Body(Vec<u8>),
    Stream(EdgeResponse),
}

#[derive(Debug)]
pub struct Response {
    body: ResponseBody,
    headers: Headers,
    status_code: u16,
}

impl Response {
    pub fn from_json<B: Serialize>(value: &B) -> Result<Self> {
        if let Ok(data) = serde_json::to_string(value) {
            return Ok(Self {
                body: ResponseBody::Body(data.into_bytes()),
                headers: Headers::new(),
                status_code: 200,
            });
        }

        Err(Error::Json(("Failed to encode data to json".into(), 500)))
    }
    pub fn ok(body: impl Into<String>) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Body(body.into().into_bytes()),
            headers: Headers::new(),
            status_code: 200,
        })
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Body(bytes),
            headers: Headers::new(),
            status_code: 200,
        })
    }
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            headers: Headers::new(),
            status_code: 200,
        })
    }
    pub fn error(msg: impl Into<String>, status: u16) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Body(msg.into().into_bytes()),
            headers: Headers::new(),
            status_code: status,
        })
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

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

    pub async fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
        serde_json::from_str(&self.text().await?).map_err(Error::from)
    }

    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(bytes.clone()),
            ResponseBody::Empty => Ok(Vec::new()),
            ResponseBody::Stream(response) => JsFuture::from(response.text()?)
                .await
                .map(|value| value.as_string().unwrap().into_bytes())
                .map_err(Error::from),
        }
    }

    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }
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
            ResponseBody::Body(mut bytes) => EdgeResponse::new_with_opt_u8_array_and_init(
                Some(&mut bytes),
                &ResponseInit {
                    status: res.status_code,
                    headers: res.headers,
                }
                .into(),
            )
            .unwrap(),
            ResponseBody::Stream(response) => response,
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
        // TODO: add logging, ideally using the log crate facade over the wasm_bindgen console.log
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
