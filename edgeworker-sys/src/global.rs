use wasm_bindgen::prelude::*;

use crate::Request;
use web_sys::RequestInit;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = EventTarget)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `EventTarget` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `EventTarget`*"]
    pub type EventTarget;

    #[wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = WorkerGlobalScope)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WorkerGlobalScope` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub type WorkerGlobalScope;

    #[wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = atob)]
    #[doc = "The `atob()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/atob)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn atob(this: &WorkerGlobalScope, atob: &str) -> Result<String, JsValue>;

    #[wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = btoa)]
    #[doc = "The `btoa()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/btoa)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn btoa(this: &WorkerGlobalScope, btoa: &str) -> Result<String, JsValue>;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Request`, `WorkerGlobalScope`*"]
    pub fn fetch_with_request(this: &WorkerGlobalScope, input: &Request) -> ::js_sys::Promise;

    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn fetch_with_str(this: &WorkerGlobalScope, input: &str) -> ::js_sys::Promise;

    #[cfg(all(feature = "Request", feature = "RequestInit",))]
    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestInit`, `WorkerGlobalScope`*"]
    pub fn fetch_with_request_and_init(
        this: &WorkerGlobalScope,
        input: &Request,
        init: &RequestInit,
    ) -> ::js_sys::Promise;

    #[cfg(feature = "RequestInit")]
    #[wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`, `WorkerGlobalScope`*"]
    pub fn fetch_with_str_and_init(
        this: &WorkerGlobalScope,
        input: &str,
        init: &RequestInit,
    ) -> ::js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
