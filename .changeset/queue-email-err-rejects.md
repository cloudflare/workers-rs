---
"workers-rs": patch
---

An `Err` returned from an `#[event(queue)]` or `#[event(email)]` handler now rejects the
handler's promise, as `#[event(connect)]` already does, instead of panicking. On the default
(panic=abort) build that panic trapped the Wasm instance, failing other requests in flight in
the isolate and reinitializing it, so static state was lost. The runtime still retries a
failed queue batch.
