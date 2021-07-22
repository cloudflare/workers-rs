mod headers;
mod router;

use std::result::Result as StdResult;

mod global;

use edgeworker_sys::{
    Cf, Request as EdgeRequest, Response as EdgeResponse, ResponseInit as EdgeResponseInit,
};
use js_sys::Date as JsDate;
use matchit::InsertError;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub use crate::headers::Headers;
pub use crate::router::Router;
use web_sys::RequestInit;

pub use edgeworker_sys::console_log;
pub use worker_kv as kv;

pub type Result<T> = StdResult<T, Error>;

pub mod prelude {
    pub use crate::global::Fetch;
    pub use crate::headers::Headers;
    pub use crate::Method;
    pub use crate::Request;
    pub use crate::Response;
    pub use crate::Result;
    pub use crate::Schedule;
    pub use crate::{Date, DateInit};
    pub use edgeworker_sys::console_log;
    pub use matchit::Params;
    pub use web_sys::RequestInit;
}

#[derive(Debug)]
pub struct Date {
    js_date: JsDate,
}

pub enum DateInit {
    Millis(u64),
    String(String),
}

impl Date {
    pub fn new(init: DateInit) -> Self {
        let val = match init {
            DateInit::Millis(n) => JsValue::from_f64(n as f64),
            DateInit::String(s) => JsValue::from_str(&s),
        };

        Self {
            js_date: JsDate::new(&val),
        }
    }

    pub fn now() -> Self {
        Self {
            js_date: JsDate::new_0(),
        }
    }

    pub fn as_millis(&self) -> u64 {
        self.js_date.get_time() as u64
    }
}

impl ToString for Date {
    fn to_string(&self) -> String {
        self.js_date.to_string().into()
    }
}
#[derive(Serialize)]
pub struct Schedule {
    event_type: String,
    time: u64,
    cron: String,
}

impl Schedule {
    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn cron(&self) -> String {
        self.cron.clone()
    }
}

impl From<(String, u64, String)> for Schedule {
    fn from(s: (String, u64, String)) -> Self {
        Self {
            event_type: s.0,
            time: s.1,
            cron: s.2,
        }
    }
}

pub struct Request {
    method: Method,
    path: String,
    headers: Headers,
    cf: Cf,
    event_type: String,
    edge_request: EdgeRequest,
    body_used: bool,
    immutable: bool,
}

impl From<(String, EdgeRequest)> for Request {
    fn from(req: (String, EdgeRequest)) -> Self {
        Self {
            method: req.1.method().into(),
            path: Url::parse(&req.1.url()).unwrap().path().into(),
            headers: Headers(req.1.headers()),
            cf: req.1.cf(),
            immutable: &req.0 == "fetch",
            event_type: req.0,
            edge_request: req.1,
            body_used: false,
        }
    }
}

impl Request {
    pub fn new(uri: &str, method: &str) -> Result<Self> {
        EdgeRequest::new_with_str_and_init(uri, RequestInit::new().method(method))
            .map(|req| (String::new(), req).into())
            .map_err(|e| {
                Error::JsError(
                    e.as_string()
                        .unwrap_or_else(|| "invalid URL or method for Request".to_string()),
                )
            })
    }

    pub fn new_with_init(uri: &str, init: &RequestInit) -> Result<Self> {
        EdgeRequest::new_with_str_and_init(uri, init)
            .map(|req| (String::new(), req).into())
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
                .and_then(|val| {
                    val.into_serde()
                        .map_err(|e| Error::RustError(e.to_string()))
                });
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

    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }

    #[allow(clippy::clippy::should_implement_trait)]
    pub fn clone(&self) -> Result<Self> {
        EdgeRequest::new_with_request(&self.edge_request)
            .map(|req| (self.event_type(), req).into())
            .map_err(Error::from)
    }

    pub fn inner(&self) -> &EdgeRequest {
        &self.edge_request
    }
}

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
    pub fn ok(body: String) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Body(body.into_bytes()),
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
    pub fn error(msg: String, status: u16) -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Body(msg.into_bytes()),
            headers: Headers::new(),
            status_code: status,
        })
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub async fn text(&mut self) -> Result<String> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(
                String::from_utf8(bytes.clone()).map_err(|e| Error::RustError(e.to_string()))?
            ),
            ResponseBody::Empty => Ok(String::new()),
            ResponseBody::Stream(response) => JsFuture::from(response.text()?)
                .await
                .map(|value| value.as_string().unwrap())
                .map_err(Error::from),
        }
    }

    pub async fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
        serde_json::from_str(&self.text().await?)
            .map_err(|_| Error::RustError("JSON deserialization error".into()))
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

impl From<Response> for EdgeResponse {
    fn from(res: Response) -> Self {
        // if let Ok(res) = EdgeResponse::new_with_opt_str(Some(res.body.as_str())) {
        //     return res;
        // }

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

impl From<worker_kv::KvError> for Error {
    fn from(e: worker_kv::KvError) -> Self {
        let val: JsValue = e.into();
        val.into()
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

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Method {
    Head = 0,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
}

impl Method {
    pub fn all() -> Vec<Method> {
        vec![
            Method::Head,
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Patch,
            Method::Delete,
            Method::Options,
            Method::Connect,
            Method::Trace,
        ]
    }
}

impl From<String> for Method {
    fn from(m: String) -> Self {
        match m.to_ascii_uppercase().as_str() {
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "OPTIONS" => Method::Options,
            "CONNECT" => Method::Connect,
            "TRACE" => Method::Trace,
            _ => Method::Get,
        }
    }
}

impl From<Method> for String {
    fn from(val: Method) -> Self {
        match val {
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Get => "GET",
        }
        .to_string()
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        (*self).clone().into()
    }
}

#[derive(Debug)]
pub enum Redirect {
    Follow,
    Error,
    Manual,
}

impl From<String> for Redirect {
    fn from(redirect: String) -> Self {
        match redirect.as_str() {
            "error" => Redirect::Error,
            "manual" => Redirect::Manual,
            _ => Redirect::Follow,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    BodyUsed,
    Json((String, u16)),
    JsError(String),
    Internal(JsValue),
    RouteInsertError(matchit::InsertError),
    RustError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BodyUsed => write!(f, "request body has already been read"),
            Error::Json((msg, status)) => write!(f, "{} (status: {})", msg, status),
            Error::JsError(s) | Error::RustError(s) => write!(f, "{}", s),
            Error::Internal(_) => write!(f, "unrecognized JavaScript object"),
            Error::RouteInsertError(e) => write!(f, "failed to insert route: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        match v
            .as_string()
            .or_else(|| v.dyn_ref::<js_sys::Error>().map(|e| e.to_string().into()))
        {
            Some(s) => Self::JsError(s),
            None => Self::Internal(v),
        }
    }
}

impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        JsValue::from_str(&e.to_string())
    }
}

impl From<InsertError> for Error {
    fn from(e: InsertError) -> Self {
        Error::RouteInsertError(e)
    }
}
