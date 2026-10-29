//! Raw import of the `cloudflare:workers` `tracing` API (custom trace spans).
//!
//! Runtime surface, from workerd's `src/cloudflare/internal/tracing.d.ts`:
//!
//! ```ts
//! declare const tracing: {
//!   enterSpan<T, A extends unknown[]>(name: string, callback: (span: Span, ...args: A) => T, ...args: A): T;
//!   startActiveSpan<T, A extends unknown[]>(name: string, callback: (span: Span, ...args: A) => T, ...args: A): T;
//!   startSpan(name: string): Span;
//!   getActiveSpan(): Span | undefined;
//! };
//! declare class Span {
//!   readonly isTraced: boolean;
//!   spanContext(): { traceId: string; spanId: string; traceFlags: number };
//!   setAttribute(key: string, value: string | number | boolean): this;
//!   setAttributes(attributes: Record<string, string | number | boolean | undefined>): this;
//!   recordException(exception: Exception): void;
//!   updateName(name: string): this;
//!   setStatus(status: { code: "unset" | "ok" | "error"; message?: string }): this;
//!   end(): void;
//! }
//! ```
//!
//! `enterSpan` is bound twice: once with a callback that returns nothing, for
//! synchronous work, and once with a callback returning a `Promise`, which
//! the runtime awaits before closing the span. Both callbacks are invoked
//! exactly once, synchronously, so the unit-returning ones can be borrowed
//! `ScopedClosure`s. `setAttribute` and `recordException` are bound once per
//! accepted argument type so the safe wrapper in `worker` stays typed.

use js_sys::Promise;
use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:workers")]
extern "C" {
    /// The `tracing` namespace object exported by `cloudflare:workers`.
    #[wasm_bindgen(thread_local_v2, js_name = tracing)]
    pub static TRACING: Tracing;

    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type Tracing;

    /// `enterSpan` with a synchronous callback: the span closes when the
    /// callback returns. The callback is invoked exactly once, before
    /// `enterSpan` returns.
    #[wasm_bindgen(method, js_name = enterSpan)]
    pub fn enter_span(this: &Tracing, name: &str, cb: &ScopedClosure<dyn FnMut(TracingSpan)>);

    /// `enterSpan` with a callback that returns a `Promise`: the runtime keeps
    /// the span open until the promise settles. Returns that same promise.
    #[wasm_bindgen(method, js_name = enterSpan)]
    pub fn enter_span_async(
        this: &Tracing,
        name: &str,
        cb: &Closure<dyn FnMut(TracingSpan) -> Promise>,
    ) -> Promise;

    /// `startActiveSpan`: the span is active while the callback runs, stays
    /// open after it returns, and is closed by an explicit `end()`.
    #[wasm_bindgen(method, js_name = startActiveSpan)]
    pub fn start_active_span(
        this: &Tracing,
        name: &str,
        cb: &ScopedClosure<dyn FnMut(TracingSpan)>,
    );

    /// `startSpan`: a span that is never the active one. Closed by `end()`.
    #[wasm_bindgen(method, js_name = startSpan)]
    pub fn start_span(this: &Tracing, name: &str) -> TracingSpan;

    /// `getActiveSpan`: the span active on the current async context, or
    /// `undefined` outside an invocation.
    #[wasm_bindgen(method, js_name = getActiveSpan)]
    pub fn get_active_span(this: &Tracing) -> Option<TracingSpan>;

    /// A span handle. Named `TracingSpan` rather than `Span` to leave the
    /// short name free for the `worker` crate's wrapper.
    #[wasm_bindgen(extends = js_sys::Object, js_name = Span)]
    #[derive(Debug, Clone)]
    pub type TracingSpan;

    #[wasm_bindgen(method, getter, js_name = isTraced)]
    pub fn is_traced(this: &TracingSpan) -> bool;

    #[wasm_bindgen(method, js_name = spanContext)]
    pub fn span_context(this: &TracingSpan) -> TracingSpanContext;

    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute_bool(this: &TracingSpan, key: &str, value: bool);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute_num(this: &TracingSpan, key: &str, value: f64);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute_str(this: &TracingSpan, key: &str, value: &str);

    /// `recordException` with a string: recorded as the exception message.
    #[wasm_bindgen(method, js_name = recordException)]
    pub fn record_exception_str(this: &TracingSpan, message: &str);

    /// `recordException` with an `Error` or `{ code?, name?, message?, stack? }`
    /// object.
    #[wasm_bindgen(method, js_name = recordException)]
    pub fn record_exception(this: &TracingSpan, exception: &JsValue);

    #[wasm_bindgen(method, js_name = updateName)]
    pub fn update_name(this: &TracingSpan, name: &str);

    /// `setStatus({ code, message? })`.
    #[wasm_bindgen(method, js_name = setStatus)]
    pub fn set_status(this: &TracingSpan, status: &JsValue);

    /// Close the span. Idempotent; spans opened with `enterSpan` close
    /// themselves and ignore this.
    #[wasm_bindgen(method, js_name = end)]
    pub fn end(this: &TracingSpan);

    /// The object returned by `Span.spanContext()`.
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type TracingSpanContext;

    #[wasm_bindgen(method, getter, js_name = traceId)]
    pub fn trace_id(this: &TracingSpanContext) -> String;
    #[wasm_bindgen(method, getter, js_name = spanId)]
    pub fn span_id(this: &TracingSpanContext) -> String;
    #[wasm_bindgen(method, getter, js_name = traceFlags)]
    pub fn trace_flags(this: &TracingSpanContext) -> u32;
}
