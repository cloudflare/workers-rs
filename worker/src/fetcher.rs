use crate::{env::EnvBinding, RequestInit, Result};
use js_sys::futures::JsFuture;
use std::convert::TryInto;
use wasm_bindgen::{JsCast, JsValue};

#[cfg(feature = "http")]
use crate::HttpResponse;
use crate::{Request, Response};
/// A struct for invoking fetch events to other Workers.
#[derive(Debug, Clone)]
pub struct Fetcher(worker_sys::Fetcher);

#[cfg(not(feature = "http"))]
type FetchResponseType = Response;
#[cfg(feature = "http")]
type FetchResponseType = HttpResponse;

impl Fetcher {
    /// Invoke a fetch event in a worker with a url and optionally a [RequestInit].
    ///
    /// Return type is [`Response`](crate::Response) unless `http` feature is enabled
    /// and then it is [`http::Response<worker::Body>`].
    pub async fn fetch(
        &self,
        url: impl Into<String>,
        init: Option<RequestInit>,
    ) -> Result<FetchResponseType> {
        let path = url.into();
        let promise = match init {
            Some(ref init) => self.0.fetch_with_str_and_init(&path, &init.into()),
            None => self.0.fetch_with_str(&path),
        }?;

        let resp_sys: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
        #[cfg(not(feature = "http"))]
        let result = Ok(Response::from(resp_sys));
        #[cfg(feature = "http")]
        let result = crate::response_from_wasm(resp_sys);
        result
    }

    /// Invoke a fetch event with an existing [Request].
    ///
    /// Argument type is [`Request`](crate::Request) or [`http::Request<worker::Body>`].
    ///
    /// Return type is [`Response`](crate::Response) unless `http` feature is enabled
    /// and then it is [`http::Response<worker::Body>`].
    pub async fn fetch_request<T, E>(&self, request: T) -> Result<FetchResponseType>
    where
        T: TryInto<Request, Error = E>,
        crate::Error: From<E>,
    {
        let req = request.try_into()?;
        let promise = self.0.fetch(req.inner())?;
        let resp_sys: web_sys::Response = JsFuture::from(promise).await?.dyn_into()?;
        let response = Response::from(resp_sys);
        #[cfg(feature = "http")]
        let result = response.try_into();
        #[cfg(not(feature = "http"))]
        let result = Ok(response);
        result
    }

    /// Convert Fetcher into user-defined RPC interface.
    /// ```
    /// # use worker::{js_sys, wasm_bindgen::{self, prelude::*}, Fetcher};
    /// #[wasm_bindgen]
    /// extern "C" {
    ///     #[wasm_bindgen(extends=js_sys::Object)]
    ///     #[derive(Debug, Clone, PartialEq, Eq)]
    ///     pub type MyRpcInterface;
    ///
    ///     #[wasm_bindgen(method, catch)]
    ///     pub fn add(
    ///         this: &MyRpcInterface,
    ///         a: u32,
    ///         b: u32,
    ///     ) -> std::result::Result<js_sys::Promise, JsValue>;
    /// }
    ///
    /// # fn example(fetcher: Fetcher) {
    /// let rpc: MyRpcInterface = fetcher.into_rpc();
    /// let result = rpc.add(1, 2);
    /// # }
    /// ```
    pub fn into_rpc<T: JsCast>(self) -> T {
        self.0.unchecked_into()
    }
}

impl EnvBinding for Fetcher {
    const TYPE_NAME: &'static str = "Fetcher";

    fn get(val: JsValue) -> Result<Self> {
        let obj = js_sys::Object::from(val);
        let name = obj.constructor().name();
        // Service bindings use ServiceStub in newer runtimes, while asset bindings
        // and older runtimes still use Fetcher.
        if name == Self::TYPE_NAME || name == "ServiceStub" {
            Ok(obj.unchecked_into())
        } else {
            Err(format!(
                "Binding cannot be cast to the type {} from {}",
                Self::TYPE_NAME,
                name
            )
            .into())
        }
    }
}

impl JsCast for Fetcher {
    fn instanceof(val: &wasm_bindgen::JsValue) -> bool {
        val.is_instance_of::<Fetcher>()
    }

    fn unchecked_from_js(val: wasm_bindgen::JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &wasm_bindgen::JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<Fetcher> for JsValue {
    fn from(service: Fetcher) -> Self {
        JsValue::from(service.0)
    }
}

impl AsRef<wasm_bindgen::JsValue> for Fetcher {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        &self.0
    }
}

impl From<worker_sys::Fetcher> for Fetcher {
    fn from(inner: worker_sys::Fetcher) -> Self {
        Self(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Env;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen(inline_js = "
        export function serviceBinding(name) {
            class Fetcher {
                fetch(url) {
                    if (url.endsWith('/fail')) return Promise.reject(new Error('service rejected'));
                    return Promise.resolve(new Response('service response', { status: 201 }));
                }
            }
            const classes = {
                Fetcher,
                ServiceStub: class ServiceStub extends Fetcher {},
                OtherBinding: class OtherBinding {},
            };
            return new classes[name]();
        }
    ")]
    extern "C" {
        #[wasm_bindgen(js_name = serviceBinding)]
        fn service_binding(name: &str) -> JsValue;
    }

    fn env_with_binding(value: &JsValue) -> Env {
        let env = js_sys::Object::new();
        js_sys::Reflect::set(&env, &"SERVICE".into(), value).unwrap();
        env.unchecked_into()
    }

    #[wasm_bindgen_test]
    fn accepts_service_stub_bindings() {
        let value = service_binding("ServiceStub");
        let env = env_with_binding(&value);
        assert_eq!(JsValue::from(env.service("SERVICE").unwrap()), value);
    }

    #[wasm_bindgen_test]
    fn accepts_legacy_fetcher_bindings() {
        let value = service_binding("Fetcher");
        let env = env_with_binding(&value);
        assert_eq!(JsValue::from(env.service("SERVICE").unwrap()), value);
        assert_eq!(JsValue::from(env.assets("SERVICE").unwrap()), value);
    }

    #[wasm_bindgen_test]
    async fn fetches_through_service_stub_bindings() {
        let env = env_with_binding(&service_binding("ServiceStub"));
        let fetcher = env.service("SERVICE").unwrap();
        let response = fetcher.fetch("https://service.test/", None).await.unwrap();
        #[cfg(not(feature = "http"))]
        {
            let mut response = response;
            assert_eq!(response.status_code(), 201);
            assert_eq!(response.text().await.unwrap(), "service response");
        }
        #[cfg(feature = "http")]
        assert_eq!(response.status(), http::StatusCode::CREATED);
        assert!(fetcher
            .fetch("https://service.test/fail", None)
            .await
            .is_err());
    }

    #[wasm_bindgen_test]
    fn rejects_unrelated_binding_types() {
        let env = env_with_binding(&service_binding("OtherBinding"));
        assert!(env
            .service("SERVICE")
            .unwrap_err()
            .to_string()
            .contains("Binding cannot be cast to the type Fetcher from OtherBinding"));
    }

    #[wasm_bindgen_test]
    fn rejects_missing_bindings() {
        let env: Env = js_sys::Object::new().unchecked_into();
        assert!(env
            .service("SERVICE")
            .unwrap_err()
            .to_string()
            .contains("Binding `SERVICE` is undefined."));
    }
}
