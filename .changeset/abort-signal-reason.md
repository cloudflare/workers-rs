---
"workers-rs": minor
---

`AbortSignal::reason()` now returns `JsValue` instead of `Option<JsValue>`, matching `web_sys::AbortSignal` and the workers-types declaration. Before the signal is aborted the reason is `undefined`; use `signal.aborted()` to check.
