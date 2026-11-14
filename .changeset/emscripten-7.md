---
"workers-rs": patch
---

Update `--emscripten` builds to Emscripten 7.0.0, whose epoll readiness listeners Tokio's hosted event loop waits on. Listener callbacks are delivered within the request that made a socket ready, so concurrent inbound connections are each served in their own request.
