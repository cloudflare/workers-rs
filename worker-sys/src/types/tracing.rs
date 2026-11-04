use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::prelude::*;

// A namespace import, so runtimes without the `tracing` export still load the module.
#[wasm_bindgen(inline_js = "
import * as workers from 'cloudflare:workers';
export function getTracing() { return workers.tracing; }
")]
extern "C" {
    /// The `tracing` export of `cloudflare:workers` (custom trace spans), if the runtime has it.
    #[wasm_bindgen(js_name = getTracing)]
    pub fn get_tracing() -> Option<Tracing>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type Tracing;

    /// `startSpan`: a child of the active span that does not itself become active.
    #[wasm_bindgen(method, js_name = startSpan)]
    pub fn start_span(this: &Tracing, name: &str) -> TracingSpan;

    /// `getActiveSpan`: `undefined` outside an invocation.
    #[wasm_bindgen(method, js_name = getActiveSpan)]
    pub fn get_active_span(this: &Tracing) -> Option<TracingSpan>;

    /// `withActiveSpan`: makes an existing span active while the callback runs, which
    /// it invokes exactly once, synchronously.
    #[wasm_bindgen(method, js_name = withActiveSpan)]
    pub fn with_active_span(this: &Tracing, span: &TracingSpan, cb: &ScopedClosure<dyn FnMut()>);

    /// The `withActiveSpan` property, to detect runtimes that predate it.
    #[wasm_bindgen(method, getter, js_name = withActiveSpan)]
    pub fn with_active_span_fn(this: &Tracing) -> JsValue;

    /// Named `TracingSpan` to leave `Span` free for the `worker` wrapper.
    #[wasm_bindgen(extends = js_sys::Object, js_name = Span)]
    #[derive(Debug, Clone)]
    pub type TracingSpan;

    #[wasm_bindgen(method, getter, js_name = isTraced)]
    pub fn is_traced(this: &TracingSpan) -> bool;

    #[wasm_bindgen(method, js_name = spanContext)]
    pub fn span_context(this: &TracingSpan) -> TracingSpanContext;

    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute(this: &TracingSpan, key: &str, value: bool);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute_with_f64(this: &TracingSpan, key: &str, value: f64);
    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute_with_str(this: &TracingSpan, key: &str, value: &str);

    #[wasm_bindgen(method, js_name = recordException)]
    pub fn record_exception_with_str(this: &TracingSpan, message: &str);
    /// `recordException` with an `Error` or a `{ code?, name?, message?, stack? }` object.
    #[wasm_bindgen(method, js_name = recordException)]
    pub fn record_exception(this: &TracingSpan, exception: &JsValue);

    #[wasm_bindgen(method, js_name = updateName)]
    pub fn update_name(this: &TracingSpan, name: &str);

    /// `setStatus({ code, message? })`.
    #[wasm_bindgen(method, js_name = setStatus)]
    pub fn set_status(this: &TracingSpan, status: &js_sys::Object);

    /// Idempotent.
    #[wasm_bindgen(method)]
    pub fn end(this: &TracingSpan);

    #[wasm_bindgen(extends = js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type TracingSpanContext;

    #[wasm_bindgen(method, getter, js_name = traceId)]
    pub fn trace_id(this: &TracingSpanContext) -> String;
    #[wasm_bindgen(method, getter, js_name = spanId)]
    pub fn span_id(this: &TracingSpanContext) -> String;
    #[wasm_bindgen(method, getter, js_name = traceFlags)]
    pub fn trace_flags(this: &TracingSpanContext) -> f64;
}
