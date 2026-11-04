//! `WorkersLayer`: a `tracing_subscriber::Layer` that records every `tracing` span as a platform
//! span and every event as attributes on the span it was emitted in.
//!
//! A `tracing` span is created and closed in two separate calls (`on_new_span` / `on_close`), so
//! the layer opens a platform span with `start_span` and ends it on close. Each platform span is
//! opened inside its `tracing` parent's platform span, so the two trees have the same shape.
//!
//! The layer cannot make a platform span active while its `tracing` span is entered, since
//! `on_enter` / `on_exit` are also separate calls. So runtime operations such as `fetch` nest under
//! the enclosing `worker::observability` span, not under a `tracing` span. Instrument a future with
//! `Span::instrument` where that matters.
//!
//! This lives in the example so `worker` does not depend on `tracing-subscriber`.

use std::cell::RefCell;
use std::collections::HashMap;

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use worker::observability::{active_span, start_span, Span};

thread_local! {
    // Registry extensions must be `Send + Sync` and a JS span is neither. A Worker isolate is
    // single-threaded, and `tracing` ids are unique among live spans.
    static OPEN: RefCell<HashMap<u64, Span>> = RefCell::new(HashMap::new());
}

fn with_span<R>(id: &Id, f: impl FnOnce(&Span) -> R) -> Option<R> {
    OPEN.with_borrow(|open| open.get(&id.into_u64()).map(f))
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WorkersLayer;

impl<S> Layer<S> for WorkersLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let name = attrs.metadata().name();
        let parent = ctx
            .span(id)
            .and_then(|span| span.parent())
            .and_then(|parent| with_span(&parent.id(), Span::clone));
        let span = match parent {
            Some(parent) => parent.in_scope(|| start_span(name)),
            None => start_span(name),
        };
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
        let visitor = |span: &Span| {
            event.record(&mut AttrVisitor {
                span,
                prefix: Some(event.metadata().level().as_str()),
            })
        };
        // Outside any `tracing` span, record onto the active platform span.
        let recorded = ctx
            .event_span(event)
            .and_then(|span| with_span(&span.id(), visitor))
            .is_some();
        if !recorded {
            if let Some(span) = active_span() {
                visitor(&span);
            }
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if let Some(span) = OPEN.with_borrow_mut(|open| open.remove(&id.into_u64())) {
            span.end();
        }
    }
}

/// Writes `tracing` fields as attributes. Event fields are prefixed with the level
/// (`INFO.message`) so they don't collide with the span's own fields.
struct AttrVisitor<'a> {
    span: &'a Span,
    prefix: Option<&'a str>,
}

impl AttrVisitor<'_> {
    fn key(&self, field: &Field) -> String {
        match self.prefix {
            Some(prefix) => format!("{prefix}.{}", field.name()),
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

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.span.record_exception(value);
        self.span.set_attribute(&self.key(field), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.span
            .set_attribute(&self.key(field), format!("{value:?}"));
    }
}
