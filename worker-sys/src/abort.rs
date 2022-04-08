use wasm_bindgen::prelude::*;

use crate::global::EventTarget;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=AbortController)]
    #[derive(Debug, PartialEq, Eq)]
    pub type AbortController;

    #[wasm_bindgen(constructor, js_class=AbortController)]
    pub fn new() -> AbortController;

    #[wasm_bindgen(structural, method, getter, js_class=AbortController, js_name=signal)]
    pub fn signal(this: &AbortController) -> AbortSignal;

    #[wasm_bindgen(method, structural, js_class=AbortController, js_name=abort)]
    pub fn abort(this: &AbortController, reason: JsValue);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = EventTarget, extends=::js_sys::Object, js_name=AbortSignal)]
    #[derive(Debug, PartialEq, Eq)]
    pub type AbortSignal;

    #[wasm_bindgen(structural, method, getter, js_class=AbortSignal, js_name=aborted)]
    pub fn aborted(this: &AbortSignal) -> bool;

    #[wasm_bindgen(structural, method, getter, js_class=AbortSignal, js_name=reason)]
    pub fn reason(this: &AbortSignal) -> JsValue;

    #[wasm_bindgen(static_method_of=AbortSignal, js_name=abort)]
    pub fn abort(reason: JsValue) -> AbortSignal;
}
