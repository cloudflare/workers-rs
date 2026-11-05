use js_sys::futures::JsFuture;
use wasm_bindgen::JsCast;

use crate::{request::Request, response::Response, AbortSignal, Result};

/// Construct a Fetch call from a URL string or a Request object. Call its `send` method to execute
/// the request.
#[derive(Debug)]
pub enum Fetch {
    Url(url::Url),
    Request(Request),
}

impl Fetch {
    /// Execute a Fetch call and receive a Response.
    ///
    /// Dropping the future before the response arrives aborts the request, so it can be
    /// cancelled with `select!` or a timeout.
    pub async fn send(&self) -> Result<Response> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), None).await,
            Fetch::Request(req) => fetch_with_request(req, None).await,
        }
    }

    /// Execute a Fetch call and receive a Response, aborted by `signal` or by dropping the
    /// future before the response arrives.
    pub async fn send_with_signal(&self, signal: &AbortSignal) -> Result<Response> {
        match self {
            Fetch::Url(url) => fetch_with_str(url.as_ref(), Some(signal)).await,
            Fetch::Request(req) => fetch_with_request(req, Some(signal)).await,
        }
    }
}

async fn fetch_with_str(url: &str, signal: Option<&AbortSignal>) -> Result<Response> {
    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let resp = fetch_abort_on_drop(None, &[signal.map(|s| &**s)], |init| {
        Ok(worker.fetch_with_str_and_init(url, init))
    })
    .await?;
    Ok(resp.into())
}

async fn fetch_with_request(request: &Request, signal: Option<&AbortSignal>) -> Result<Response> {
    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let req = request.inner();
    let req_signal = req.signal();
    let resp = fetch_abort_on_drop(None, &[Some(&req_signal), signal.map(|s| &**s)], |init| {
        Ok(worker.fetch_with_request_and_init(req, init))
    })
    .await?;
    Ok(resp.into())
}

/// Runs a fetch with `init` (or an empty one) carrying a signal that aborts it if the returned
/// future is dropped before the response arrives, combined with `signals`. A settled fetch is not
/// aborted, as that would also cancel its response body.
pub(crate) async fn fetch_abort_on_drop(
    init: Option<web_sys::RequestInit>,
    signals: &[Option<&web_sys::AbortSignal>],
    fetch: impl FnOnce(&web_sys::RequestInit) -> Result<js_sys::Promise>,
) -> Result<web_sys::Response> {
    let controller = AbortOnDrop(Some(web_sys::AbortController::new()?));
    let own = controller.0.as_ref().unwrap().signal();
    let all = js_sys::Array::of1(&own);
    for signal in signals.iter().flatten() {
        all.push(signal);
    }
    let signal = if all.length() == 1 {
        own
    } else {
        web_sys::AbortSignal::any(&all)
    };
    let init = init.unwrap_or_default();
    init.set_signal(Some(&signal));
    let resp = JsFuture::from(fetch(&init)?).await;
    controller.disarm();
    Ok(resp?.dyn_into()?)
}

struct AbortOnDrop(Option<web_sys::AbortController>);

impl AbortOnDrop {
    fn disarm(mut self) {
        self.0 = None;
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        if let Some(controller) = self.0.take() {
            controller.abort();
        }
    }
}
