//! Custom trace spans for Workers Observability.
//!
//! Wraps the `cloudflare:workers` [custom-span API][api]: [`enter_span`] (sync)
//! and [`enter_span_async`] (for `async` handlers) open a span that appears in
//! the trace waterfall alongside the automatic `fetch` / KV / D1 spans, with
//! correct parent-child nesting. Attach metadata with [`Span::set_attribute`].
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
//! Spans are **callback-scoped**: a span opens when `enter_span` is called and
//! closes when the callback returns (sync) or its future resolves (async).
//! There is no imperative start/end — the platform measures duration itself,
//! which is why durations are accurate even though guest timer resolution is
//! clamped.
//!
//! ## Bridging the `tracing` crate
//!
//! [`with_active_span`] exposes the innermost open span so a
//! `tracing_subscriber::Layer` can forward `tracing` events/fields onto it as
//! attributes. A ready-made layer isn't included here to keep this crate free
//! of a `tracing-subscriber` dependency — see the `custom-spans` example for a
//! `WorkersLayer` you can copy.
//!
//! [api]: https://developers.cloudflare.com/changelog/post/2026-06-16-custom-spans/

use std::cell::RefCell;
use std::future::Future;

use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

use crate::bindings::tracing as raw;

/// A handle to an open span. Cheap to clone (a refcounted JS object). Valid
/// only while the span that produced it is open, which by construction is the
/// body of the [`enter_span`] / [`enter_span_async`] call that yielded it.
#[derive(Debug, Clone)]
pub struct Span(raw::Span);

impl Span {
    /// Attach an attribute. Accepts any [`AttributeValue`] (`bool`, integer,
    /// float, or string).
    pub fn set_attribute(&self, key: &str, value: impl AttributeValue) {
        value.set_on(self, key);
    }

    /// Whether this request is being sampled. Use it to skip building
    /// expensive attributes when the span won't be recorded.
    pub fn is_traced(&self) -> bool {
        self.0.is_traced()
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A value that can be attached to a [`Span`] — the JS `boolean | number |
/// string` union. Integers are widened to `f64` (JS `number`); values past
/// 2^53 lose precision. Sealed: implemented for the primitive types only.
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

thread_local! {
    /// Stack of currently-open platform spans, innermost last. Pushed by
    /// `enter_span` / `enter_span_async` for the duration of their body so a
    /// `tracing` layer (see [`with_active_span`]) can target the active span. A
    /// thread-local rather than the `tracing` registry because a JS [`Span`] is
    /// `!Send` and registry extensions must be `Send + Sync`.
    static ACTIVE: RefCell<Vec<Span>> = const { RefCell::new(Vec::new()) };
}

fn push_active(span: &Span) {
    ACTIVE.with_borrow_mut(|s| s.push(span.clone()));
}

fn pop_active() {
    ACTIVE.with_borrow_mut(|s| {
        s.pop();
    });
}

/// Run `f` with the innermost open span, if any; returns `None` when no span is
/// open. The hook a `tracing_subscriber::Layer` uses to forward events onto the
/// active platform span (see the `custom-spans` example). `f` must not re-enter
/// this function — a borrow is held for its duration.
pub fn with_active_span<R>(f: impl FnOnce(&Span) -> R) -> Option<R> {
    ACTIVE.with_borrow(|stack| stack.last().map(f))
}

/// Open a synchronous custom span named `name`, run `f` inside it, and close
/// it when `f` returns. Nests under any span already open on the JS async
/// context — including an enclosing [`enter_span`] — so the platform tree
/// mirrors the call tree.
pub fn enter_span<T>(name: &str, f: impl FnOnce(&Span) -> T) -> T {
    let mut f = Some(f);
    let mut out: Option<T> = None;

    // `enterSpan` invokes this exactly once, synchronously, before returning,
    // so the callback may borrow non-`'static` state. `borrow_mut` encodes
    // that the JS function must not outlive this stack frame.
    let mut cb = |span: raw::Span| {
        let span = Span(span);
        push_active(&span);
        let f = f.take().expect("enterSpan invoked its callback twice");
        out = Some(f(&span));
        pop_active();
    };

    {
        let scoped = ScopedClosure::borrow_mut(&mut cb);
        raw::TRACING.with(|t| t.enter_span_sync(name, &scoped));
    }

    out.expect("enterSpan must invoke its callback synchronously")
}

/// Open an asynchronous custom span named `name`, drive `f`'s future inside it,
/// and close it when the future resolves. This is the form `async` request
/// handlers need.
///
/// Unlike [`enter_span`], the callback returns a `Promise` that outlives the
/// `enterSpan` call (workerd awaits it), so `f` and its future must be
/// `'static`.
pub async fn enter_span_async<F, Fut, T>(name: &str, f: F) -> T
where
    F: FnOnce(Span) -> Fut + 'static,
    Fut: Future<Output = T> + 'static,
    T: 'static,
{
    let result: std::rc::Rc<RefCell<Option<T>>> = std::rc::Rc::new(RefCell::new(None));
    let sink = result.clone();

    let mut f = Some(f);
    let cb = Closure::wrap(Box::new(move |span: raw::Span| -> js_sys::Promise {
        let span = Span(span);
        let f = f.take().expect("enterSpan invoked its callback twice");
        let fut = f(span.clone());
        let sink = sink.clone();
        future_to_promise(async move {
            push_active(&span);
            let value = fut.await;
            pop_active();
            *sink.borrow_mut() = Some(value);
            Ok(JsValue::UNDEFINED)
        })
    }) as Box<dyn FnMut(raw::Span) -> js_sys::Promise>);

    let promise = raw::TRACING.with(|t| t.enter_span_async(name, &cb));
    // Keep `cb` alive until the span's promise settles, then drop it.
    let _ = JsFuture::from(promise).await;
    drop(cb);

    std::rc::Rc::try_unwrap(result)
        .ok()
        .expect("span future outlived its handle")
        .into_inner()
        .expect("enterSpan async callback must resolve the result")
}
