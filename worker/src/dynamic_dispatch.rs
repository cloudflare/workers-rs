use wasm_bindgen::{JsCast, JsValue};
use worker_sys::DynamicDispatcher as DynamicDispatcherSys;

use crate::{env::EnvBinding, Fetcher, Result};

/// A binding for dispatching events to Workers inside of a dispatch namespace by their name. This
/// allows for your worker to directly invoke many workers by name instead of having multiple
/// service worker bindings.
///
/// # Example:
///
/// ```no_run
/// # use js_sys::Object;
/// # use wasm_bindgen::JsCast;
/// # let env: worker::Env = Object::new().unchecked_into();
/// # tokio_test::block_on(async {
/// # #[cfg(feature="http")]
/// # let req = http::Request::get("http://localhost:8080").body(worker::Body::empty())?;
/// # #[cfg(not(feature="http"))]
/// # let req = worker::Request::new("http://localhost:8080", worker::Method::Get)?;
/// let dispatcher = env.dynamic_dispatcher("DISPATCHER")?;
/// let fetcher = dispatcher.get("namespaced-worker-name")?;
/// let resp = fetcher.fetch_request(req).await?;
/// # Ok::<(), worker::Error>(())
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct DynamicDispatcher(DynamicDispatcherSys);

impl DynamicDispatcher {
    /// Gets a [Fetcher] for a Worker inside of the dispatch namespace based of the name specified.
    pub fn get(&self, name: impl Into<String>) -> Result<Fetcher> {
        let fetcher_sys = self.0.get(name.into(), JsValue::undefined())?;
        Ok(fetcher_sys.into())
    }
}

impl EnvBinding for DynamicDispatcher {
    const TYPE_NAME: &'static str = "DynamicDispatcher";
}

impl JsCast for DynamicDispatcher {
    fn instanceof(val: &JsValue) -> bool {
        val.is_string()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        DynamicDispatcher(val.unchecked_into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for DynamicDispatcher {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        &self.0
    }
}

impl From<JsValue> for DynamicDispatcher {
    fn from(val: JsValue) -> Self {
        DynamicDispatcher(val.unchecked_into())
    }
}

impl From<DynamicDispatcher> for JsValue {
    fn from(sec: DynamicDispatcher) -> Self {
        sec.0.into()
    }
}
