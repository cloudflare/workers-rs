use crate::error::Error;
use crate::headers::Headers;
use crate::http::Method;
use crate::Result;

use edgeworker_sys::{Cf, Request as EdgeRequest};
use serde::de::DeserializeOwned;
use url::Url;
use web_sys::RequestInit;

pub struct Request {
    method: Method,
    url: String,
    path: String,
    headers: Headers,
    cf: Cf,
    edge_request: EdgeRequest,
    body_used: bool,
    immutable: bool,
}

impl From<EdgeRequest> for Request {
    fn from(req: EdgeRequest) -> Self {
        Self {
            method: req.method().into(),
            url: req.url(),
            path: Url::parse(&req.url())
                .map(|u| u.path().into())
                .unwrap_or_else(|_| {
                    let u = req.url();
                    if !u.starts_with('/') {
                        return "/".to_string() + &u;
                    }
                    u
                }),
            headers: Headers(req.headers()),
            cf: req.cf(),
            edge_request: req,
            body_used: false,
            immutable: true,
        }
    }
}

impl Request {
    pub fn new(uri: &str, method: &str) -> Result<Self> {
        EdgeRequest::new_with_str_and_init(uri, RequestInit::new().method(method))
            .map(|req| {
                let mut req: Request = req.into();
                req.immutable = false;
                req
            })
            .map_err(|e| {
                Error::JsError(
                    e.as_string()
                        .unwrap_or_else(|| "invalid URL or method for Request".to_string()),
                )
            })
    }

    pub fn new_with_init(uri: &str, init: &RequestInit) -> Result<Self> {
        EdgeRequest::new_with_str_and_init(uri, init)
            .map(|req| {
                let mut req: Request = req.into();
                req.immutable = false;
                req
            })
            .map_err(|e| {
                Error::JsError(
                    e.as_string()
                        .unwrap_or_else(|| "invalid URL or options for Request".to_string()),
                )
            })
    }

    pub async fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
        if !self.body_used {
            self.body_used = true;
            return wasm_bindgen_futures::JsFuture::from(EdgeRequest::json(&self.edge_request)?)
                .await
                .map_err(|e| {
                    Error::JsError(
                        e.as_string()
                            .unwrap_or_else(|| "failed to get JSON for body value".into()),
                    )
                })
                .and_then(|val| val.into_serde().map_err(Error::from));
        }

        Err(Error::BodyUsed)
    }

    pub async fn text(&mut self) -> Result<String> {
        if !self.body_used {
            self.body_used = true;
            return wasm_bindgen_futures::JsFuture::from(EdgeRequest::text(&self.edge_request)?)
                .await
                .map(|val| val.as_string().unwrap())
                .map_err(|e| {
                    Error::JsError(
                        e.as_string()
                            .unwrap_or_else(|| "failed to get text for body value".into()),
                    )
                });
        }

        Err(Error::BodyUsed)
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    // Headers can only be modified if the request was created from scratch or cloned
    pub fn headers_mut(&mut self) -> Result<&mut Headers> {
        if self.immutable {
            return Err(Error::JsError(
                "Cannot get a mutable reference to an immutable headers object.".into(),
            ));
        }
        Ok(&mut self.headers)
    }

    pub fn cf(&self) -> Cf {
        self.cf.clone()
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(&self) -> Result<Self> {
        self.edge_request
            .clone()
            .map(|req| req.into())
            .map_err(Error::from)
    }

    pub fn inner(&self) -> &EdgeRequest {
        &self.edge_request
    }
}
