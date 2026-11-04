---
"workers-rs": minor
---

Add `worker::observability`, bindings for Workers [custom trace spans](https://developers.cloudflare.com/workers/observability/traces/custom-spans/). `enter_span` / `enter_span_async` run code inside a new span, `start_span` opens one without making it active, and `Span::in_scope` / `Span::instrument` make a span active for a closure or a future, re-entering it on every poll so concurrent futures nest correctly. Every handler and `wait_until` future now runs under the span active when it started. Nesting needs a runtime with `tracing.withActiveSpan`; on older runtimes spans are still recorded but nest under the enclosing span.
