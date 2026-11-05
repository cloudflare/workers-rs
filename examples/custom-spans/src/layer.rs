//! `WorkersLayer` — a `tracing_subscriber::Layer` that turns every
//! `tracing::span!` into a real Workers platform span and forwards `tracing`
//! events onto it.
//!
//! ## Why it's shaped this way
//!
//! `tracing` models a span lifetime as two separate operations: the span is
//! created (`on_new_span`) and closed (`on_close`) at unrelated points in time.
//! The original custom-span API was callback-scoped only (`enterSpan(name,
//! cb)`), which a `Layer` can't drive: at `on_new_span` it would have to call
//! `enterSpan` and not return from its callback until the later `on_close`,
//! which a single-threaded Worker can't suspend and resume.
//!
//! [`startActiveSpan()` + `span.end()`][changelog] (2026-07-28) are exactly the
//! imperative pair that lifetime needs, so this layer now bridges span
//! *lifetimes*, not just events: `#[tracing::instrument]` and `span!` produce
//! spans in the trace waterfall with platform-measured durations, with no
//! Workers-specific code at the call site.
//!
//! ## The one limitation
//!
//! Parent-child nesting follows the platform's async context, which is only
//! entered for the instant `startActiveSpan` runs its callback. A bridged span
//! therefore parents under the nearest enclosing span opened by
//! [`worker::observability::enter_span`] / `enter_span_async`, and two nested
//! `tracing` spans come out as siblings under that same parent rather than one
//! inside the other. Wrap a subtree in `enter_span` where the shape matters;
//! closing the gap entirely needs a runtime primitive for attaching to an open
//! span's context.
//!
//! This lives in the example rather than the `worker` crate so `worker` stays
//! free of a `tracing-subscriber` dependency. Copy it into your project, or
//! lift it into `worker` behind a feature if your project wants it there.
//!
//! [changelog]: https://developers.cloudflare.com/changelog/post/2026-07-28-start-active-span/

use std::cell::RefCell;
use std::collections::HashMap;

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use worker::observability::{start_active_span, with_active_span, Span};

thread_local! {
    /// Platform span per live `tracing` span id. A thread-local map rather than
    /// the registry's own span extensions because a JS [`Span`] is `!Send` and
    /// extensions must be `Send + Sync`; a Worker isolate is single-threaded,
    /// so a thread-local is equivalent here. Entries are removed in `on_close`,
    /// which `tracing` calls exactly once per span.
    static OPEN: RefCell<HashMap<u64, Span>> = RefCell::new(HashMap::new());
}

/// Bridges `tracing` onto Workers Observability: each `tracing` span becomes a
/// platform span for its full lifetime, and events land as attributes on the
/// span they were emitted in. Install it on a `tracing_subscriber` registry.
#[derive(Debug, Default, Clone, Copy)]
pub struct WorkersLayer;

impl<S> Layer<S> for WorkersLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let span = start_active_span(attrs.metadata().name());
        attrs.record(&mut AttrVisitor {
            span: &span,
            prefix: None,
        });
        OPEN.with_borrow_mut(|open| open.insert(id.into_u64(), span));
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, _ctx: Context<'_, S>) {
        with_span(id, |span| {
            values.record(&mut AttrVisitor { span, prefix: None })
        });
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Events belong to the span they were emitted in; fall back to the
        // innermost `enter_span` when there is no enclosing `tracing` span.
        let level = event.metadata().level().as_str();
        let recorded = ctx.event_span(event).is_some_and(|s| {
            with_span(&s.id(), |span| {
                event.record(&mut AttrVisitor {
                    span,
                    prefix: Some(level),
                })
            })
            .is_some()
        });

        if !recorded {
            with_active_span(|span| {
                event.record(&mut AttrVisitor {
                    span,
                    prefix: Some(level),
                })
            });
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if let Some(span) = OPEN.with_borrow_mut(|open| open.remove(&id.into_u64())) {
            span.end();
        }
    }
}

/// Run `f` against the platform span backing `id`, if it is still open.
fn with_span<R>(id: &Id, f: impl FnOnce(&Span) -> R) -> Option<R> {
    OPEN.with_borrow(|open| open.get(&id.into_u64()).map(f))
}

/// Writes each visited `tracing` field as a typed `setAttribute` on the
/// platform span. Span fields keep their own names; event fields are prefixed
/// with the level (`"INFO.message"`) so they don't collide with them.
struct AttrVisitor<'a> {
    span: &'a Span,
    prefix: Option<&'a str>,
}

impl AttrVisitor<'_> {
    fn key(&self, field: &Field) -> String {
        match self.prefix {
            Some(prefix) => format!("{}.{}", prefix, field.name()),
            None => field.name().to_owned(),
        }
    }
}

impl Visit for AttrVisitor<'_> {
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.span.set_attribute(&self.key(field), value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.span.set_attribute(&self.key(field), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.span.set_attribute(&self.key(field), value);
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.span.set_attribute(&self.key(field), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.span.set_attribute(&self.key(field), value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.span
            .set_attribute(&self.key(field), format!("{value:?}").as_str());
    }
}
