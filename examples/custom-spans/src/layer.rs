//! `WorkersLayer` — a `tracing_subscriber::Layer` that forwards `tracing`
//! events and span fields onto the active Workers platform span.
//!
//! ## Why it's shaped this way
//!
//! `tracing` models a span lifetime as two separate operations — `on_enter`
//! and a later `on_exit`, driven by a guard's `Drop`. The platform models it
//! as one **callback-scoped** operation (`enterSpan(name, cb)`); there is no
//! imperative start/end. A `Layer` therefore can't bridge an arbitrary
//! `tracing::span!` to a platform span: at `on_enter` it would have to call
//! `enterSpan` and not return from its callback until the separate `on_exit`,
//! which a single-threaded Worker can't suspend and resume. Durations have to
//! come from the platform anyway, since guest timer resolution is clamped.
//!
//! So the work splits: span **structure + timing** come from wrapping work in
//! [`worker::observability::enter_span`] / `enter_span_async` (closure-scoped,
//! so they map onto `enterSpan` and nest via the JS async context); **events +
//! fields** are forwarded by this layer onto the active span via
//! [`worker::observability::with_active_span`].
//!
//! This lives in the example rather than the `worker` crate so `worker` stays
//! free of a `tracing-subscriber` dependency. Copy it into your project, or
//! lift it into `worker` behind a feature if your project wants it there.

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use worker::observability::{with_active_span, Span};

/// Forwards `tracing` events and span fields onto the active platform span as
/// attributes. Install it on a `tracing_subscriber` registry and establish
/// platform spans with `worker::observability::enter_span[_async]`.
#[derive(Debug, Default, Clone, Copy)]
pub struct WorkersLayer;

impl<S> Layer<S> for WorkersLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let prefix = attrs.metadata().name();
        with_active_span(|span| attrs.record(&mut AttrVisitor { span, prefix }));
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let prefix = ctx.span(id).map(|s| s.name()).unwrap_or("span");
        with_active_span(|span| values.record(&mut AttrVisitor { span, prefix }));
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let prefix = event.metadata().level().as_str();
        with_active_span(|span| event.record(&mut AttrVisitor { span, prefix }));
    }
}

/// Writes each visited `tracing` field as a typed `setAttribute` on the
/// platform span, under the key `"<prefix>.<field>"`.
struct AttrVisitor<'a> {
    span: &'a Span,
    prefix: &'a str,
}

impl AttrVisitor<'_> {
    fn key(&self, field: &Field) -> String {
        format!("{}.{}", self.prefix, field.name())
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
