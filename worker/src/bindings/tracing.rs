//! Raw `wasm-bindgen` import of the `cloudflare:workers` `tracing` API.
//!
//! Hand-written (not `ts-gen`-generated) because the safe wrapper in
//! [`crate::tracing`] needs a couple of shapes `ts-gen` doesn't express well —
//! the two `enterSpan` callback forms (sync vs. promise-returning) and the
//! `setAttribute` value overloads.
//!
//! Platform surface (`@cloudflare/workers-types`):
//!
//! ```ts
//! interface Tracing {
//!   enterSpan<T, A extends unknown[]>(
//!     name: string,
//!     callback: (span: Span, ...args: A) => T,
//!     ...args: A
//!   ): T;
//! }
//! declare abstract class Span {
//!   get isTraced(): boolean;
//!   setAttribute(key: string, value?: boolean | number | string): void;
//! }
//! ```

use js_sys::Promise;
use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:workers")]
extern "C" {
    /// The `tracing` namespace object exported by `cloudflare:workers`.
    /// Thread-local because a Worker isolate is single-threaded and the
    /// binding is not `Sync`.
    #[wasm_bindgen(thread_local_v2, js_name = tracing)]
    pub(crate) static TRACING: Tracing;

    #[wasm_bindgen(js_name = Tracing)]
    pub(crate) type Tracing;

    /// `enterSpan` with a synchronous callback: the span closes when the
    /// callback returns.
    #[wasm_bindgen(method, js_name = enterSpan)]
    pub(crate) fn enter_span_sync(this: &Tracing, name: &str, cb: &ScopedClosure<dyn FnMut(Span)>);

    /// `enterSpan` with an async callback: the callback returns a `Promise`
    /// and workerd keeps the span open until it settles. `enterSpan` returns
    /// that same promise.
    #[wasm_bindgen(method, js_name = enterSpan)]
    pub(crate) fn enter_span_async(
        this: &Tracing,
        name: &str,
        cb: &Closure<dyn FnMut(Span) -> Promise>,
    ) -> Promise;

    /// A live span handle. Refcounted JS object — cloning is cheap and a clone
    /// stays valid while the span is open.
    #[wasm_bindgen(js_name = Span)]
    #[derive(Debug, Clone)]
    pub(crate) type Span;

    // `setAttribute(key, value)` is `boolean | number | string` in JS; bind one
    // overload per kind so the safe wrapper stays typed.
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub(crate) fn set_attribute_bool(this: &Span, key: &str, value: bool);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub(crate) fn set_attribute_num(this: &Span, key: &str, value: f64);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub(crate) fn set_attribute_str(this: &Span, key: &str, value: &str);

    #[wasm_bindgen(method, getter, js_name = isTraced)]
    pub(crate) fn is_traced(this: &Span) -> bool;
}
