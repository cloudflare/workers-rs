# ts-gen-worker

Cloudflare Workers bindings for Rust generated from TypeScript declarations by
[ts-gen](https://github.com/wasm-bindgen/ts-gen).

Unlike the `worker` crate, the bindings are not hand-written: `src/bindings.rs`
is the generated `wasm-bindgen` projection of `@cloudflare/workers-types`
(`node_modules/@cloudflare/workers-types/index.d.ts`), restricted to the
declarations `worker` uses through `--export <Name>` roots listed in
`chompfile.toml` (ts-gen emits each root plus everything it references).
Upstream declarations carry no Rust-specific tags, so `types/annotations.d.ts`
re-declares only the members that need one (`@throws`, and `@rust` integer
types where `worker` exposes a JS number as `u16`/`u32`/`usize`) and ts-gen
layers them on by name (`--annotate`). Apps generate their own `Env` the same
way from their bindings (see `test/types/env.d.ts`). What a `.d.ts` cannot
express, the Rust idioms, lives in `src/ext/` as `impl` blocks on the generated
types: `#[event]` handler exports, serde, futures over promises, `Method`,
header iteration, `ByteStream`, `Router`, `#[durable_object]`, `console_log!`,
`Delay`, `Cors`, `Fetch`, `EnvExt`, the KV builders and `send`.

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
            Response::builder().status(201).from_json(&user)
        })
        .run(req, env)
        .await
}
```

Generic upstream types (`Request<CfHostMetadata, Cf>`, `ReadableStream<R>`)
keep their parameters with defaults, so the bare names work everywhere.

Build with `cargo build --target wasm32-unknown-unknown` and
`wasm-bindgen --target module --force-enable-abort-handler`; the output's
default export is the worker. The bindings use js-sys's unstable API, so the
build needs `--cfg js_sys_unstable_apis` (`.cargo/config.toml` rustflags for
`wasm32-unknown-unknown`, or `RUSTFLAGS`). The flag lets the glue notice a trap (a panic
under `panic=abort`, `unreachable`, out of memory) and reinstantiate the
module before the next call, so one failed request does not poison the
isolate. `#[durable_object]` classes carry
`#[wasm_bindgen(reinit = "reconstruct")]`, so an instance the runtime
constructed before the trap is rebuilt from its `(state, env)` on its next
call instead of being rejected as stale. The attribute is wasm-bindgen PR
[#5375](https://github.com/wasm-bindgen/wasm-bindgen/pull/5375), unreleased:
the workspace builds from the `wasm-bindgen/` submodule, which carries it and
the generic-mono changes in `parity/mono-prototype-patches.md`, through
`[patch.crates-io]`, and `ts-gen-worker` cannot publish before those ship.

## Errors

Every binding that can throw returns a `Result`; there is no panicking
variant. Methods tagged `@throws` in `types/annotations.d.ts` return the
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
See `MIGRATING.md` for the guide.

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

- `chomp build:ts-gen-worker:types` regenerates `src/bindings.rs` from
  workers-types (edit the `--export` roots in `chompfile.toml` to widen the
  surface, `types/annotations.d.ts` to tag members) and `test/src/env.rs`
  from `test/types/env.d.ts`, with `--experimental-generic-mono`.
- The ts-gen-worker build tasks set `--cfg js_sys_unstable_apis` through
  `RUSTFLAGS` and build into `target/mono`, so the rest of the workspace
  (`worker`) builds without it. Without the cfg the crates are empty on the
  host and a compile error on wasm32. Under it, `serde-wasm-bindgen` and
  `wasm-streams` build from the patched copies in `vendor-patches/`.
  `chomp lint:ts-gen-worker` runs clippy on the four crates with the cfg.
- `chomp test:ts-gen-worker` builds the test worker and runs its vitest suite
  under miniflare; `chomp test:ts-gen-worker-http` runs it with the handler
  over the `http` types. `test/` is the `worker` crate's suite ported to the
  generated API.
