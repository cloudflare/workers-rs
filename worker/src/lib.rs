use std::{iter::{FromIterator, Map}, ops::Deref, result::Result as StdResult};

use edgeworker_sys::{
    Cf, Request as EdgeRequest, Response as EdgeResponse, ResponseInit as EdgeResponseInit, Headers as EdgeHeaders
};
use js_sys::{Array, JsString};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use wasm_bindgen::JsValue;

pub use worker_kv as kv;

pub type Result<T> = StdResult<T, Error>;

pub mod prelude {
    pub use crate::Method;
    pub use crate::Request;
    pub use crate::Response;
    pub use crate::Result;
    pub use crate::Schedule;
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

pub struct Headers (EdgeHeaders);

#[allow(clippy::new_without_default)]
impl Headers {
    pub fn new() -> Self {
        Headers (EdgeHeaders::new().unwrap())
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).unwrap()
    }

    pub fn has(&self, name: &str) -> bool {
        self.0.has(name).unwrap()
    }

    pub fn append(&mut self, name: &str, value: &str) {
        self.0.append(name, value).unwrap()
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.0.set(name, value).unwrap()
    }

    pub fn delete(&mut self, name: &str) {
        self.0.delete(name).unwrap()
    }

    pub fn entries(&self) -> HeaderIterator {
        self.0.entries().unwrap().into_iter()
            .map((|a| a.unwrap().into()) as F1)
            .map(|a: Array| (a.get(0).as_string().unwrap(), a.get(1).as_string().unwrap()))
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.0.keys().unwrap().into_iter()
            .map(|a| a.unwrap().as_string().unwrap())
    }

    pub fn values(&self) -> impl Iterator<Item = String> {
        self.0.values().unwrap().into_iter()
            .map(|a| a.unwrap().as_string().unwrap())
    }
}

type F1 = fn(StdResult<JsValue, JsValue>) -> Array;
type HeaderIterator = Map<Map<js_sys::IntoIter, F1>, fn(Array) -> (String, String)>;

impl IntoIterator for &Headers {
    type Item = (String, String);

    type IntoIter = HeaderIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.entries()
    }
}

impl <T: Deref<Target=str>> FromIterator<(T, T)> for Headers {
    fn from_iter<U: IntoIterator<Item = (T, T)>>(iter: U) -> Self {
        let mut headers = Headers::new();
        for (name, value) in iter {
            headers.set(&name, &value);
        }
        headers
    }
}

impl <'a, T: Deref<Target=str>> FromIterator<&'a (T, T)> for Headers {
    fn from_iter<U: IntoIterator<Item = &'a (T, T)>>(iter: U) -> Self {
        let mut headers = Headers::new();
        iter.into_iter().for_each(|(name, value)| {headers.set(name, value)});
        headers
    }
}

impl Clone for Headers {
    fn clone(&self) -> Self {
        Headers (EdgeHeaders::new_with_headers(&self.0).unwrap())
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
            headers: Headers (req.1.headers()),
            cf: req.1.cf(),
            event_type: req.0,
            edge_request: req.1,
            body_used: false,
        }
    }
}

impl Request {
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

    pub fn set_headers(&mut self, headers: Headers) {
        self.headers = headers
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
                headers: res.headers
            }
            .into(),
        )
        .unwrap()

        // TODO: add logging, ideally using the log crate facade over the wasm_bindgen console.log
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
    pub headers: Headers
}

impl From<ResponseInit> for EdgeResponseInit {
    fn from(init: ResponseInit) -> Self {
        let mut edge_init = EdgeResponseInit::new();
        edge_init.status(init.status);
        edge_init.headers(&init.headers.0);
        edge_init
    }
}

#[derive(Debug, Clone)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
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

pub enum Error {
    BodyUsed,
    Json((String, u16)),
    JsError(String),
    Internal(JsValue),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::BodyUsed => "request body has already been read".into(),
            Error::Json((msg, status)) => format!("{} (status: {})", msg, status),
            Error::JsError(s) => s.clone(),
            Error::Internal(v) => JsString::from(v.clone()).into(),
        }
    }
}

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
