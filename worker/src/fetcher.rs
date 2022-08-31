use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use worker_sys::{Fetcher as FetcherSys, Response as ResponseSys};

use crate::{Request, RequestInit, Response, Result};

/// A struct for invoking fetch events to other Workers.
pub struct Fetcher(FetcherSys);

impl Fetcher {
    /// Invoke a fetch event in a worker with a path and optionally a [RequestInit].
    pub async fn fetch(
        &self,
        path: impl Into<String>,
        init: Option<RequestInit>,
    ) -> Result<Response> {
        let path = path.into();
        let promise = match init {
            Some(ref init) => self.0.fetch_with_str_and_init(&path, &init.into()),
            None => self.0.fetch_with_str(&path),
        };

        let resp_sys: ResponseSys = JsFuture::from(promise).await?.dyn_into()?;
        Ok(Response::from(resp_sys))
    }

    /// Invoke a fetch event with an existing [Request].
    pub async fn fetch_request(&self, request: Request) -> Result<Response> {
        let promise = self.0.fetch(request.inner());
        let resp_sys: ResponseSys = JsFuture::from(promise).await?.dyn_into()?;
        Ok(Response::from(resp_sys))
    }
}

impl From<FetcherSys> for Fetcher {
    fn from(inner: FetcherSys) -> Self {
        Self(inner)
    }
}
