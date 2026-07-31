---
"workers-rs": minor
---

Add `State::block_concurrency_while` and `State::block_concurrency_while_infallible` for
Durable Objects, binding the runtime's `blockConcurrencyWhile`. Both run an async closure
while blocking delivery of other events until it completes; the `_infallible` variant
returns application errors as values instead of resetting the object.
