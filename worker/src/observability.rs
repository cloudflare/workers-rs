//! Custom trace spans for Workers Observability.
//!
//! Binds the `tracing` namespace of `cloudflare:workers` ([custom spans][api]):
//! spans you open here appear in the trace waterfall next to the automatic
//! `fetch` / KV / D1 spans, nested under whatever span is active on the JS
//! async context when they are opened, with durations measured by the
//! platform.
//!
//! ```ignore
//! use worker::observability::{enter_span, enter_span_async};
//!
//! # async fn handler() {
//! enter_span_async("handle_request", |span| async move {
//!     span.set_attribute("http.path", "/items");
//!     enter_span("load_rows", |child| {
//!         child.set_attribute("db.rows", 42);
//!     });
//! })
//! .await;
//! # }
//! ```
//!
//! Four ways to open a span, mirroring the runtime API:
//!
//! - [`enter_span`] / [`enter_span_async`]: the span is active for the body
//!   and closes when the body returns (or its future resolves). Use these for
//!   most instrumentation.
//! - [`start_active_span`]: active for the body, but stays open after it
//!   returns until [`Span::end`]. For work that outlives a callback, such as a
//!   streamed response body.
//! - [`start_span`]: never active, closed by [`Span::end`]. For timing an
//!   operation nothing else needs to nest under.
//!
//! [`active_span`] returns the span active on the current async context, so
//! you can annotate it without threading a handle through your code.
//!
//! Spans are recorded only when tracing is enabled for the Worker
//! (`observability.traces.enabled` in `wrangler.toml`) and the invocation is
//! sampled; otherwise every call here is a cheap no-op and
//! [`Span::is_traced`] returns `false`.
//!
//! ## Nesting and Rust futures
//!
//! The platform derives parent/child relationships from the JS async context,
//! which it propagates through promise continuations. A Rust future is polled
//! from the `wasm-bindgen-futures` executor, so work inside an `.await` is
//! not reliably inside the span that was active when the future was created.
//! [`enter_span_async`] runs its body inside the span's own callback, which
//! gives the best alignment the runtime offers today, but spans opened after
//! an `.await` can still attach to a different parent than you expect.
//! Prefer [`enter_span`] for synchronous subtrees where the shape matters.
//!
//! ## Bridging the `tracing` crate
//!
//! [`start_span`] + [`Span::end`] map onto `tracing`'s two-phase span
//! lifetime, so a `tracing_subscriber::Layer` can turn every `tracing::span!`
//! into a platform span with a platform-measured duration, and [`active_span`]
//! lets it attach events that fire outside any `tracing` span. The crate ships
//! no such layer, to stay free of a `tracing-subscriber` dependency; the
//! `custom-spans` example contains a `WorkersLayer` you can copy.
//!
//! [api]: https://developers.cloudflare.com/workers/observability/traces/custom-spans/

use std::cell::RefCell;
use std::fmt::Display;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use worker_sys::{TracingSpan, TRACING};

/// The JS callback shape for the promise-returning `enterSpan`. (The
/// synchronous callbacks are spelled out inline: a `dyn` alias defaults to
/// `'static`, and those closures borrow their caller's frame.)
type AsyncCallback = dyn FnMut(TracingSpan) -> js_sys::Promise;

/// A handle to a span. Cheap to clone: a refcounted JS object. Calls on a
/// span that has already ended are ignored by the runtime.
#[derive(Debug, Clone)]
pub struct Span(TracingSpan);

impl Span {
    /// Attach an attribute. Accepts any [`AttributeValue`]: `bool`, integers,
    /// floats, and strings. Returns `self` so calls can be chained.
    pub fn set_attribute(&self, key: &str, value: impl AttributeValue) -> &Self {
        value.set_on(self, key);
        self
    }

    /// Whether this span will be recorded: `false` when tracing is disabled,
    /// the invocation is not sampled, or the span has already ended. Use it
    /// to skip building expensive attributes.
    pub fn is_traced(&self) -> bool {
        self.0.is_traced()
    }

    /// The span's identity: trace ID, span ID, and W3C trace flags. Still
    /// valid after the span ends. When observability is disabled for the
    /// Worker the IDs are all zeros and must not be propagated.
    pub fn span_context(&self) -> SpanContext {
        let ctx = self.0.span_context();
        SpanContext {
            trace_id: ctx.trace_id(),
            span_id: ctx.span_id(),
            trace_flags: ctx.trace_flags() as u8,
        }
    }

    /// Record an exception event on the span, with `exception`'s `Display`
    /// output as the message. Accepts `&str`, `String`, and any error type.
    /// Does not end the span or change its status; see [`Span::set_status`].
    pub fn record_exception(&self, exception: &(impl Display + ?Sized)) {
        self.0.record_exception_str(&exception.to_string());
    }

    /// Record a JS exception on the span: an `Error` (recorded with its
    /// `name`, `message`, and `stack`) or an object with at least one of
    /// `code`, `name`, or `message`.
    pub fn record_js_exception(&self, exception: &JsValue) {
        self.0.record_exception(exception);
    }

    /// Set the span's status. [`SpanStatus::Ok`] is final: later calls are
    /// ignored. Returns `self` so calls can be chained.
    pub fn set_status(&self, status: SpanStatus) -> &Self {
        let (code, message) = match status {
            SpanStatus::Ok => ("ok", None),
            SpanStatus::Error { message } => ("error", message),
        };
        let status = js_sys::Object::new();
        // `Reflect::set` on a fresh plain object cannot fail.
        let _ = js_sys::Reflect::set(&status, &"code".into(), &code.into());
        if let Some(message) = message {
            let _ = js_sys::Reflect::set(&status, &"message".into(), &message.into());
        }
        self.0.set_status(&status);
        self
    }

    /// Rename the span, for when the right name is only known after it
    /// started (a matched route, say). Names are truncated to 64 bytes.
    /// Returns `self` so calls can be chained.
    pub fn update_name(&self, name: &str) -> &Self {
        self.0.update_name(name);
        self
    }

    /// Close a span opened by [`start_active_span`] or [`start_span`].
    /// Idempotent, so a span whose end is driven by racing events (a stream
    /// that either completes or is cancelled) can end from both. A no-op on
    /// a span from [`enter_span`] / [`enter_span_async`], which the runtime
    /// closes itself.
    pub fn end(&self) {
        self.0.end();
    }
}

/// The identity of a [`Span`], as returned by [`Span::span_context`]. The
/// required fields of an OpenTelemetry `SpanContext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanContext {
    /// 32 lowercase hex characters; shared by every span in the invocation.
    pub trace_id: String,
    /// 16 lowercase hex characters.
    pub span_id: String,
    /// W3C trace flags. Does not currently indicate whether the invocation is
    /// sampled; use [`Span::is_traced`] for that.
    pub trace_flags: u8,
}

/// A span status, following the OpenTelemetry model. Every span starts
/// unset; see [`Span::set_status`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    /// The operation succeeded. Final once set.
    Ok,
    /// The operation failed, with an optional developer-facing message.
    Error {
        /// A description of the failure.
        message: Option<String>,
    },
}

impl SpanStatus {
    /// An error status carrying `message`.
    pub fn error(message: impl Into<String>) -> Self {
        SpanStatus::Error {
            message: Some(message.into()),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A value that can be attached to a [`Span`]: the JS `boolean | number |
/// string` union. Integers are widened to `f64` (a JS `number`), so values
/// beyond 2^53 lose precision. Sealed; implemented for the primitive types
/// and for strings.
pub trait AttributeValue: sealed::Sealed {
    #[doc(hidden)]
    fn set_on(self, span: &Span, key: &str);
}

impl sealed::Sealed for bool {}
impl AttributeValue for bool {
    fn set_on(self, span: &Span, key: &str) {
        span.0.set_attribute_bool(key, self);
    }
}

impl sealed::Sealed for &str {}
impl AttributeValue for &str {
    fn set_on(self, span: &Span, key: &str) {
        span.0.set_attribute_str(key, self);
    }
}

impl sealed::Sealed for String {}
impl AttributeValue for String {
    fn set_on(self, span: &Span, key: &str) {
        span.0.set_attribute_str(key, &self);
    }
}

impl sealed::Sealed for &String {}
impl AttributeValue for &String {
    fn set_on(self, span: &Span, key: &str) {
        span.0.set_attribute_str(key, self);
    }
}

macro_rules! attribute_value_num {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl AttributeValue for $t {
            fn set_on(self, span: &Span, key: &str) {
                span.0.set_attribute_num(key, self as f64);
            }
        }
    )*};
}
attribute_value_num!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Open a span named `name`, run `f` inside it, and close it when `f`
/// returns. The span is the active one while `f` runs, so spans and platform
/// operations inside `f` nest under it.
pub fn enter_span<T>(name: &str, f: impl FnOnce(&Span) -> T) -> T {
    let mut f = Some(f);
    let mut out = None;

    // `enterSpan` invokes the callback exactly once, synchronously, before it
    // returns, so the callback may borrow this frame's state and the JS
    // function can be scoped to it. Unwind safety: if `f` panics, the panic
    // propagates out of `enterSpan` and this frame, and `out` is never read.
    let mut cb = |span: TracingSpan| {
        let f = f.take().expect("enterSpan invoked its callback twice");
        out = Some(f(&Span(span)));
    };
    {
        let scoped =
            ScopedClosure::<dyn FnMut(TracingSpan)>::borrow_mut_assert_unwind_safe(&mut cb);
        TRACING.with(|tracing| tracing.enter_span(name, &scoped));
    }

    out.expect("enterSpan must invoke its callback synchronously")
}

/// Open a span named `name`, drive the future `f` returns inside it, and
/// close it when that future resolves. The form `async` handlers need.
///
/// The future runs inside the span's JS callback (as a promise the runtime
/// awaits before closing the span), so `f` and its future must be `'static`. Work inside it
/// nests under the span as far as the JS async context follows the future's
/// polls; see the module docs.
pub async fn enter_span_async<F, Fut, T>(name: &str, f: F) -> T
where
    F: FnOnce(Span) -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
    T: 'static,
{
    let slot: Rc<RefCell<Option<T>>> = Rc::default();
    let mut f = Some(f);

    // The callback still runs exactly once, synchronously, but wasm-bindgen
    // only hands a value-returning closure to JS as an owned, `'static` one,
    // so this one owns its captures and is dropped once `enterSpan` returns.
    // Unwind safety: a panic in `f` or its future surfaces as a rejected
    // promise and is re-raised below; `slot` is only read after that.
    let promise = {
        let slot = slot.clone();
        let cb = Closure::<AsyncCallback>::wrap_assert_unwind_safe(Box::new(
            move |span: TracingSpan| -> js_sys::Promise {
                let f = f.take().expect("enterSpan invoked its callback twice");
                let fut = f(Span(span));
                let slot = slot.clone();
                future_to_promise(AssertUnwindSafe(async move {
                    *slot.borrow_mut() = Some(fut.await);
                    Ok(JsValue::UNDEFINED)
                }))
            },
        ));
        TRACING.with(|tracing| tracing.enter_span_async(name, &cb))
    };

    let settled = JsFuture::from(promise).await;
    let value = slot.borrow_mut().take();
    match (value, settled) {
        (Some(value), _) => value,
        // Only reachable when the body panicked and the panic was caught at
        // the JS boundary (`--panic-unwind`): the promise rejected with it.
        (None, Err(err)) => panic!(
            "enter_span_async: the span body did not complete: {}",
            crate::Error::from(err)
        ),
        (None, Ok(_)) => {
            panic!("enter_span_async: enterSpan settled before the span body completed")
        }
    }
}

/// Open a span named `name` and run `f` with it active, but leave the span
/// open after `f` returns: you close it with [`Span::end`]. For work that
/// extends past a callback, such as a response body that streams after the
/// handler has returned. `f` receives an owned handle so it can be moved
/// into whatever will end it.
///
/// Once `f` returns the span is no longer the active parent even though it is
/// open, so later spans will not nest under it; nest under it from inside
/// `f`. Forgetting [`Span::end`] leaves the span open until the invocation
/// finishes, when the runtime submits it with a meaningless duration.
pub fn start_active_span<T>(name: &str, f: impl FnOnce(Span) -> T) -> T {
    let mut f = Some(f);
    let mut out = None;

    // Same synchronous, invoked-once contract and unwind-safety argument as
    // `enter_span`.
    let mut cb = |span: TracingSpan| {
        let f = f
            .take()
            .expect("startActiveSpan invoked its callback twice");
        out = Some(f(Span(span)));
    };
    {
        let scoped =
            ScopedClosure::<dyn FnMut(TracingSpan)>::borrow_mut_assert_unwind_safe(&mut cb);
        TRACING.with(|tracing| tracing.start_active_span(name, &scoped));
    }

    out.expect("startActiveSpan must invoke its callback synchronously")
}

/// Open a span named `name` under the active span without making it active,
/// and return it. Close it with [`Span::end`]. Nothing nests under a span
/// opened this way; spans opened while it is open become its siblings. For
/// timing an operation, and for bridging a span lifetime that is created and
/// closed in two separate calls, as `tracing_subscriber::Layer` does.
pub fn start_span(name: &str) -> Span {
    Span(TRACING.with(|tracing| tracing.start_span(name)))
}

/// The span active on the current async context: the innermost open
/// [`enter_span`] / [`start_active_span`] body, or the invocation's root span
/// outside any custom span. `None` outside an invocation, such as at module
/// top level, or when execution has detached from the request's async
/// context.
pub fn active_span() -> Option<Span> {
    TRACING.with(|tracing| tracing.get_active_span()).map(Span)
}
