use wasm_bindgen::prelude::*;

use crate::cache::Caches;
use crate::{Request, RequestInit};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = EventTarget)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `EventTarget` class."]
    pub type EventTarget;

    #[wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = WorkerGlobalScope)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WorkerGlobalScope` class."]
    pub type WorkerGlobalScope;

    #[wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = atob)]
    #[doc = "The `atob()` method."]
    pub fn atob(this: &WorkerGlobalScope, atob: &str) -> Result<String, JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = btoa)]
    #[doc = "The `btoa()` method."]
    pub fn btoa(this: &WorkerGlobalScope, btoa: &str) -> Result<String, JsValue>;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    pub fn fetch_with_request(this: &WorkerGlobalScope, input: &Request) -> ::js_sys::Promise;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    pub fn fetch_with_str(this: &WorkerGlobalScope, input: &str) -> ::js_sys::Promise;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    pub fn fetch_with_request_and_init(
        this: &WorkerGlobalScope,
        input: &Request,
        init: &RequestInit,
    ) -> ::js_sys::Promise;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    pub fn fetch_with_str_and_init(
        this: &WorkerGlobalScope,
        input: &str,
        init: &RequestInit,
    ) -> ::js_sys::Promise;

    #[wasm_bindgen(js_name = setTimeout)]
    pub fn set_timeout(closure: &Closure<dyn FnMut()>, millis: u32) -> u32;

    #[wasm_bindgen(js_name = clearTimeout)]
    pub fn clear_timeout(id: u32);

    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(s: &str);

    #[wasm_bindgen(method, structural, getter, js_class = "WorkerGlobalScope")]
    #[doc = "Provides access to the [caches](https://developers.cloudflare.com/workers/runtime-apis/cache) global"]
    pub fn caches(this: &WorkerGlobalScope) -> Caches;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}
