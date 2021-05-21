use std::result::Result as StdResult;

use edgeworker_sys::{
    Cf, Request as EdgeRequest, Response as EdgeResponse, ResponseInit as EdgeResponseInit,
};
use js_sys::JsString;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use wasm_bindgen::JsValue;

pub use worker_kv as kv;

pub type Result<T> = StdResult<T, Error>;

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
    cf: Cf,
    event_type: String,
    edge_request: EdgeRequest,
    body_used: bool,
}

impl From<(String, EdgeRequest)> for Request {
    fn from(req: (String, EdgeRequest)) -> Self {
        Self {
            method: EdgeRequest::method(&req.1).into(),
            path: Url::parse(&EdgeRequest::url(&req.1)).unwrap().path().into(),
            cf: EdgeRequest::cf(&req.1),
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
    status_code: u16,
}

impl Response {
    pub fn json<B: Serialize>(value: &B) -> Result<Self> {
        if let Ok(data) = serde_json::to_string(value) {
            return Ok(Self {
                body: Some(data),
                status_code: 200,
            });
        }

        Err(Error::Json(("Failed to encode data to json".into(), 500)))
    }
    pub fn ok(body: Option<String>) -> Result<Self> {
        Ok(Self {
            body,
            status_code: 200,
        })
    }

    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: None,
            status_code: 200,
        })
    }

    pub fn error(msg: String, status: u16) -> Result<Self> {
        Ok(Self {
            body: Some(msg),
            status_code: status,
        })
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
}

impl From<ResponseInit> for EdgeResponseInit {
    fn from(init: ResponseInit) -> Self {
        let mut edge_init = EdgeResponseInit::new();
        edge_init.status(init.status);
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
