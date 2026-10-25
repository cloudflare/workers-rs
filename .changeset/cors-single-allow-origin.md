---
"workers-rs": patch
---

Fix `Cors::with_origins` sending several origins as a comma-separated
`Access-Control-Allow-Origin` value, which browsers never match. Add
`Cors::with_request_origin` to pass the request's `Origin` header: with several origins the
matching one is sent, together with `Vary: Origin`. A single origin or `*` is sent as before,
without `Vary`.

With several origins, `Cors::apply_headers` now returns an error when `with_request_origin` was
not called, for example `cors.with_request_origin(req.headers().get("Origin")?)`.
