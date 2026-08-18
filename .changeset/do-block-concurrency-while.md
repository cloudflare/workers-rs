---
"workers-rs": minor
---

Add `State::block_concurrency_while` for Durable Objects, binding the runtime's
`blockConcurrencyWhile` with the same call-time semantics as JavaScript: the event delivery
gate closes eagerly at the call, and the returned future yields the closure's value when
awaited. Discarding the future instead supports the constructor idiom, where `new()` gates
delivery of all events until async initialization completes. A closure returning `Err`
rejects the promise and resets the object, matching a JavaScript throw; return errors inside
`Ok` to propagate them as values.
