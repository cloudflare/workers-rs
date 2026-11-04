---
"workers-rs": minor
---

Durable Objects can now use `http` types. `DurableObject` takes the `fetch` request and response types as optional type parameters (`DurableObject<Req = Request, Res = Response>`), accepting any `FromRequest` / `IntoResponse` types like the `#[event(fetch)]` handler, so with the `http` feature a Durable Object can pass an `http::Request<worker::Body>` straight to an `axum::Router`. Existing `impl DurableObject for T` blocks are unchanged. Also adds `Stub::fetch_request`, which mirrors `Fetcher::fetch_request` and returns `http::Response<worker::Body>` when the `http` feature is enabled.
