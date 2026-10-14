# ts-gen-worker

Cloudflare Workers bindings for Rust generated from TypeScript declarations by
[ts-gen](https://github.com/wasm-bindgen/ts-gen).

Unlike the `worker` crate, the bindings are not hand-written: `types/worker.d.ts`
declares the Workers surface in TypeScript and `src/bindings.rs` is its
generated `wasm-bindgen` projection. Apps generate their own `Env` the same
way from their bindings (see `test/types/env.d.ts`). What a `.d.ts` cannot
express, the Rust idioms, lives in `src/ext/` as `impl` blocks on the generated
types: `#[event]` handler exports, serde, futures over promises, `Method`,
header iteration, `ByteStream`, `Router`, `#[durable_object]`.

## Usage

```rust
use ts_gen_worker::*;

mod env; // generated from your Env .d.ts

#[event(fetch)]
async fn main(req: Request, env: env::Env, ctx: ExecutionContext) -> Result<Response> {
    Router::<env::Env>::new()
        .get("/", |_req, _ctx| Response::builder().ok("Hello from Rust!"))
        .post_async("/user/:id", |req, ctx| async move {
            let user: User = req.json_into().await?;
            Response::builder().status(201.0).from_json(&user)
        })
        .run(req, env)
        .await
}
```

Build with `cargo build --target wasm32-unknown-unknown` and
`wasm-bindgen --target module`; the output's default export is the worker.

## Errors

Every binding that can throw returns a `Result`; there is no panicking
variant. Methods documented with `@throws` in `types/worker.d.ts` return the
generated `JsException` enum, which has one variant per JS error class named
anywhere in the declarations plus `Other(JsValue)`, so the class can be
matched directly:

```rust
match headers.set("bad name", "x") {
    Err(JsException::TypeError(e)) => log(&e.message()),
    Err(other) => return Err(other.into()),
    Ok(()) => {}
}
```

Everything else returns `ts_gen_worker::Error`, which wraps `JsException` as
`Error::Js` (and `Error::Rust(String)` for Rust-side failures), converts
from and into `JsValue`, and is what a handler returns. `Result<T>` is
`Result<T, Error>`.

Generate app `Env` bindings with the same policy so their calls `?` into the
same type: `ts-gen --fallible --error-type ts_gen_worker::Error`.

Generated names follow the JS names. Where `worker` had a shortcut, the
mapping is: `Response::ok(..)` → `Response::builder().ok(..)` (and `error`,
`empty`, `from_json`, `from_html`, `from_bytes`, `from_stream` likewise, after any
`status(..)` / `headers(..)` on the builder), `req.json()` → `req.json_into()`,
`ctx.wait_until(fut)` → `ctx.spawn(fut)`, `req.method()` → `req.method().parse::<Method>()`.
See `PARITY.md` for the full table.

## `http` feature

With `http` enabled a `fetch` handler can take and return the `http` crate's
types, with `ByteStream` as the body:

```rust
#[event(fetch)]
async fn main(req: HttpRequest, env: env::Env, ctx: ExecutionContext) -> Result<HttpResponse> {
    ..
}
```

`Request`/`Response` and `http::Request`/`http::Response<B>` convert with
`try_into()` in both directions; `AbortSignal`, `IncomingRequestCfProperties`
and `WebSocket` travel in the `http` extensions. The feature also adds
`HeaderMap` conversions on `Headers`.

## Development

- `chomp build:ts-gen-worker:types` regenerates `src/bindings.rs` and
  `test/src/env.rs` after editing the `.d.ts` files.
- `chomp test:ts-gen-worker` builds the test worker and runs its vitest suite
  under miniflare; `chomp test:ts-gen-worker-http` runs it with the handler
  over the `http` types.
