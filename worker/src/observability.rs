//! Custom trace spans for Workers Observability.
//!
//! Binds the `tracing` export of `cloudflare:workers` ([custom spans][api]). Spans opened here
//! appear in the trace waterfall next to the automatic `fetch` / KV / D1 spans, with durations
//! measured by the platform, and are recorded only when tracing is enabled for the Worker
//! (`observability.traces.enabled` in `wrangler.toml`) and the invocation is sampled.
//!
//! ```ignore
//! use worker::observability::{enter_span, enter_span_async};
//!
//! enter_span_async("handle_request", |span| async move {
//!     span.set_attribute("http.path", "/items");
//!     let rows = env.d1("DB")?.prepare("SELECT ...").all().await?; // nests under the span
//!     enter_span("render", |child| {
//!         child.set_attribute("rows", rows.len());
//!     });
//! })
//! .await;
//! ```
//!
//! ## Nesting
//!
//! The platform parents each new span, and each runtime span such as `fetch`, under the span
//! active on the JS async context. Rust futures are polled by an executor that runs every pending
//! task from one shared microtask, so that context is not the task's own. The spans here therefore
//! follow the [`tracing::Instrument`] model: a span is made active again on every poll of the
//! future it instruments ([`Span::instrument`]), or for the duration of a closure
//! ([`Span::in_scope`]). Concurrent futures each nest under their own span.
//!
//! [api]: https://developers.cloudflare.com/workers/observability/traces/custom-spans/
//! [`tracing::Instrument`]: https://docs.rs/tracing/latest/tracing/trait.Instrument.html

use std::fmt::Display;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::JsValue;
use worker_sys::{TracingSpan, TRACING};

/// A handle to a span. Cheap to clone. Calls on a span that has ended are ignored. Two handles
/// are equal when they refer to the same span.
#[derive(Debug, Clone)]
pub struct Span(TracingSpan);

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        AsRef::<JsValue>::as_ref(&self.0) == AsRef::<JsValue>::as_ref(&other.0)
    }
}

impl Eq for Span {}

impl Span {
    /// Run `f` with this span active: spans and runtime operations started inside `f` nest under
    /// it. Does not end the span.
    pub fn in_scope<R>(&self, f: impl FnOnce() -> R) -> R {
        let mut f = Some(f);
        let mut out = None;
        // `withActiveSpan` invokes the callback once, synchronously. The callback must not unwind
        // into JS: a panic would reach the closure glue instead of the export boundary. So the
        // unwind is caught here and resumed once `withActiveSpan` has returned, and `f` behaves as
        // if called directly. Without `panic = "unwind"` this is a plain call.
        let mut cb = || {
            let f = f.take().expect("callback invoked twice");
            out = Some(std::panic::catch_unwind(AssertUnwindSafe(f)));
        };
        let cb = ScopedClosure::<dyn FnMut()>::borrow_mut_aborting(&mut cb);
        TRACING.with(|tracing| tracing.with_active_span(&self.0, &cb));
        drop(cb);
        match out.expect("withActiveSpan invokes its callback synchronously") {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    /// Wrap `future` so this span is active on every poll. Does not end the span.
    pub fn instrument<F: Future>(&self, future: F) -> Instrumented<F> {
        Instrumented {
            span: Some(self.clone()),
            inner: future,
        }
    }

    /// Attach an attribute: a `bool`, number, or string. Integers become a JS `number`, so values
    /// beyond 2^53 lose precision.
    pub fn set_attribute(&self, key: &str, value: impl AttributeValue) -> &Self {
        value.set_on(&self.0, key);
        self
    }

    /// Whether this span will be recorded: `false` when tracing is disabled, the invocation is
    /// not sampled, or the span has ended. Use it to skip building expensive attributes.
    pub fn is_traced(&self) -> bool {
        self.0.is_traced()
    }

    /// The span's identity, still valid after it ends. All zeros when tracing is disabled, in
    /// which case it must not be propagated.
    pub fn span_context(&self) -> SpanContext {
        let ctx = self.0.span_context();
        SpanContext {
            trace_id: ctx.trace_id(),
            span_id: ctx.span_id(),
            trace_flags: ctx.trace_flags() as u8,
        }
    }

    /// Record an exception event with `exception`'s `Display` output as the message. Does not
    /// change the span's status.
    pub fn record_exception(&self, exception: &(impl Display + ?Sized)) {
        self.0.record_exception_with_str(&exception.to_string());
    }

    /// Record a JS exception: an `Error`, or an object with a `code`, `name`, or `message`.
    pub fn record_js_exception(&self, exception: &JsValue) {
        self.0.record_exception(exception);
    }

    /// Set the span's status. [`SpanStatus::Ok`] is final.
    pub fn set_status(&self, status: SpanStatus) -> &Self {
        let (code, message) = match status {
            SpanStatus::Unset => ("unset", None),
            SpanStatus::Ok => ("ok", None),
            SpanStatus::Error(message) => ("error", message),
        };
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"code".into(), &code.into());
        if let Some(message) = message {
            let _ = js_sys::Reflect::set(&obj, &"message".into(), &message.into());
        }
        self.0.set_status(&obj);
        self
    }

    /// Rename the span, e.g. once the matched route is known.
    pub fn update_name(&self, name: impl AsRef<str>) -> &Self {
        self.0.update_name(name.as_ref());
        self
    }

    /// End the span. Idempotent.
    pub fn end(&self) {
        self.0.end();
    }
}

/// A future with a [`Span`] active on every poll, from [`Span::instrument`] or
/// [`in_current_span`].
#[pin_project]
#[derive(Debug)]
pub struct Instrumented<F> {
    span: Option<Span>,
    #[pin]
    inner: F,
}

impl<F: Future> Future for Instrumented<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        let this = self.project();
        match this.span {
            // The executor's microtask usually already carries this span, as continuations
            // registered during the previous poll captured it, so re-entering is skipped then.
            Some(span) if !worker_sys::is_active_span(&span.0) => {
                span.in_scope(|| this.inner.poll(cx))
            }
            _ => this.inner.poll(cx),
        }
    }
}

/// The identity of a [`Span`]: the required fields of an OpenTelemetry `SpanContext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanContext {
    /// 32 lowercase hex characters, shared by every span in the invocation.
    pub trace_id: String,
    /// 16 lowercase hex characters.
    pub span_id: String,
    /// W3C trace flags.
    pub trace_flags: u8,
}

/// A span status, following the OpenTelemetry model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    Unset,
    Ok,
    /// A failure, with an optional developer-facing message.
    Error(Option<String>),
}

mod sealed {
    pub trait Sealed {}
}

/// A value accepted by [`Span::set_attribute`]: the JS `boolean | number | string` union.
pub trait AttributeValue: sealed::Sealed {
    #[doc(hidden)]
    fn set_on(self, span: &TracingSpan, key: &str);
}

impl sealed::Sealed for bool {}
impl AttributeValue for bool {
    fn set_on(self, span: &TracingSpan, key: &str) {
        span.set_attribute(key, self);
    }
}

impl sealed::Sealed for &str {}
impl AttributeValue for &str {
    fn set_on(self, span: &TracingSpan, key: &str) {
        span.set_attribute_with_str(key, self);
    }
}

impl sealed::Sealed for String {}
impl AttributeValue for String {
    fn set_on(self, span: &TracingSpan, key: &str) {
        span.set_attribute_with_str(key, &self);
    }
}

impl sealed::Sealed for &String {}
impl AttributeValue for &String {
    fn set_on(self, span: &TracingSpan, key: &str) {
        span.set_attribute_with_str(key, self);
    }
}

macro_rules! attribute_value_num {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl AttributeValue for $t {
            fn set_on(self, span: &TracingSpan, key: &str) {
                span.set_attribute_with_f64(key, self as f64);
            }
        }
    )*};
}
attribute_value_num!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Open a span under the active span, without making it active. End it with [`Span::end`]; make
/// it active with [`Span::in_scope`] or [`Span::instrument`].
pub fn start_span(name: &str) -> Span {
    Span(TRACING.with(|tracing| tracing.start_span(name)))
}

/// The span active on the current async context, or the invocation's root span outside any
/// custom span. `None` outside an invocation.
pub fn active_span() -> Option<Span> {
    TRACING.with(|tracing| tracing.get_active_span()).map(Span)
}

/// Wrap `future` so the span active now is active on every poll. Every `#[event]` handler,
/// Durable Object method, and `wait_until` future is wrapped this way, so code outside any custom
/// span sees the span it started under rather than whichever span the executor last polled in.
pub fn in_current_span<F: Future>(future: F) -> Instrumented<F> {
    Instrumented {
        span: active_span(),
        inner: future,
    }
}

/// Ends the span when dropped, so a panic or a cancelled future still closes it.
struct EndOnDrop(Span);

impl Drop for EndOnDrop {
    fn drop(&mut self) {
        self.0.end();
    }
}

/// Run `f` inside a new span named `name`, and end the span when `f` returns.
pub fn enter_span<T>(name: &str, f: impl FnOnce(&Span) -> T) -> T {
    let guard = EndOnDrop(start_span(name));
    guard.0.in_scope(|| f(&guard.0))
}

/// Run the future `f` returns inside a new span named `name`, and end the span when it
/// completes or is dropped.
pub async fn enter_span_async<F, Fut>(name: &str, f: F) -> Fut::Output
where
    F: FnOnce(Span) -> Fut,
    Fut: Future,
{
    let guard = EndOnDrop(start_span(name));
    let future = guard.0.in_scope(|| f(guard.0.clone()));
    guard.0.instrument(future).await
}
