mod headers;

use std::result::Result as StdResult;

mod global;

use edgeworker_sys::{
    Cf, Request as EdgeRequest, Response as EdgeResponse, ResponseInit as EdgeResponseInit,
};
use js_sys::{Date as JsDate, JsString};
use matchit::{InsertError, Match, Node, Params};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use wasm_bindgen::JsValue;

pub use crate::headers::Headers;
use web_sys::RequestInit;
pub use global::fetch_with_str;

pub use worker_kv as kv;

pub type Result<T> = StdResult<T, Error>;

pub mod prelude {
    pub use crate::headers::Headers;
    pub use crate::Method;
    pub use crate::Request;
    pub use crate::Response;
    pub use crate::Result;
    pub use crate::Schedule;
    pub use crate::{Date, DateInit};
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

pub type HandlerFn = fn(Request, Params) -> Result<Response>;
pub type HandlerSet = [Option<HandlerFn>; 9];

pub struct Router {
    handlers: matchit::Node<HandlerSet>,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, func, vec![Method::Get])
    }

    pub fn post(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, func, vec![Method::Post])
    }

    pub fn on(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, func, Method::all())
    }

    fn add_handler(&mut self, pattern: &str, func: HandlerFn, methods: Vec<Method>) -> Result<()> {
        // Did some testing and it appears as though a pattern can always match itself
        // i.e. the path "/user/:id" will always match the pattern "/user/:id"
        if let Ok(Match {
            value: handler_set,
            params: _,
        }) = self.handlers.at_mut(pattern)
        {
            for method in methods {
                handler_set[method as usize] = Some(func);
            }
        } else {
            let mut handler_set = [None; 9];
            for method in methods {
                handler_set[method as usize] = Some(func);
            }
            self.handlers.insert(pattern, handler_set)?;
        }

        Ok(())
    }

    pub fn run(&self, req: Request) -> Result<Response> {
        if let Ok(Match { value, params }) = self.handlers.at(&req.path()) {
            if let Some(handler) = value[req.method() as usize] {
                return (handler)(req, params);
            }
            return Response::error("Method Not Allowed".into(), 405);
        }
        Response::error("Not Found".into(), 404)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self {
            handlers: Node::new(),
        }
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
}

impl From<(String, EdgeRequest)> for Request {
    fn from(req: (String, EdgeRequest)) -> Self {
        Self {
            method: req.1.method().into(),
            path: Url::parse(&req.1.url()).unwrap().path().into(),
            headers: Headers(req.1.headers()),
            cf: req.1.cf(),
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
                .map(|val| val.into_serde().unwrap())
                .map_err(|e| {
                    Error::JsError(
                        e.as_string()
                            .unwrap_or_else(|| "failed to get JSON for body value".into()),
                    )
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

    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
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
        self.edge_request
            .clone()
            .map(|req| (self.event_type(), req).into())
            .map_err(|e| {
                if let Some(s) = e.as_string() {
                    Error::JsError(s)
                } else {
                    Error::BodyUsed
                }
            })
    }

    pub fn inner(&self) -> &EdgeRequest {
        &self.edge_request
    }
}

pub struct Response {
    body: Option<String>,
    headers: Headers,
    status_code: u16,
}

impl Response {
    pub fn json<B: Serialize>(value: &B) -> Result<Self> {
        if let Ok(data) = serde_json::to_string(value) {
            return Ok(Self {
                body: Some(data),
                headers: Headers::new(),
                status_code: 200,
            });
        }

        Err(Error::Json(("Failed to encode data to json".into(), 500)))
    }
    pub fn ok(body: Option<String>) -> Result<Self> {
        Ok(Self {
            body,
            headers: Headers::new(),
            status_code: 200,
        })
    }
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: None,
            headers: Headers::new(),
            status_code: 200,
        })
    }
    pub fn error(msg: String, status: u16) -> Result<Self> {
        Ok(Self {
            body: Some(msg),
            headers: Headers::new(),
            status_code: status,
        })
    }

    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    pub fn set_headers(&mut self, headers: Headers) {
        self.headers = headers
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

        EdgeResponse::new_with_opt_str_and_init(
            res.body.as_deref(),
            &ResponseInit {
                status: res.status_code,
                headers: res.headers,
            }
            .into(),
        )
        .unwrap()

        // TODO: add logging, ideally using the log crate facade over the wasm_bindgen console.log
    }
}

impl From<EdgeResponse> for Response {
    fn from(res: EdgeResponse) -> Self {
        Self {
            body: None,
            headers: Headers::new(),
            status_code: res.status(),
        }
    }
}

impl From<worker_kv::KvError> for Error {
    fn from(e: worker_kv::KvError) -> Self {
        let val: JsValue = e.into();
        Error::Internal(val)
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

#[derive(Debug)]
pub enum Error {
    BodyUsed,
    Json((String, u16)),
    JsError(String),
    Internal(JsValue),
    RouteInsertError(matchit::InsertError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BodyUsed => write!(f, "request body has already been read"),
            Error::Json((msg, status)) => write!(f, "{} (status: {})", msg, status),
            Error::JsError(s) => write!(f, "{}", s),
            Error::Internal(v) => {
                let s: String = JsString::from(v.clone()).into();
                write!(f, "{}", s)
            }
            Error::RouteInsertError(e) => write!(f, "failed to insert route: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        Error::JsError(
            v.as_string()
                .unwrap_or_else(|| "Failed to convert value to error.".into()),
        )
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
