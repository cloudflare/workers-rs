---
"workers-rs": minor
---

Add `Queue::metrics()`, binding the runtime's `Queue.metrics()`. It returns a `QueueMetrics`
with the backlog's message count, its size in bytes, and the timestamp of its oldest message
(`None` when the backlog is empty), so producers can apply backpressure.
