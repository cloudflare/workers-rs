# ts-gen-worker

Cloudflare Workers bindings for Rust generated from TypeScript declarations by
[ts-gen](https://github.com/wasm-bindgen/ts-gen).

Unlike the `worker` crate, nothing here is hand-written: `types/worker.d.ts`
declares the Workers surface in TypeScript and `src/bindings.rs` is its
generated `wasm-bindgen` projection. Apps generate their own `Env` the same
way from their bindings (see `test/types/env.d.ts`).

## Usage

```rust
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

mod env; // generated from your Env .d.ts

#[wasm_bindgen(js_name = "fetch", js_namespace = "default")]
pub async fn fetch(req: Request, env: env::Env, _ctx: ExecutionContext) -> Result<Response> {
    let headers = Headers::new()?;
    headers.set("content-type", "text/plain")?;
    Ok(Response::new_with_str_and_init(
        &format!("{} {}", req.method(), req.url()),
        &ResponseInit::builder().headers(&headers).build(),
    )?)
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

## Development

- `chomp build:ts-gen-worker:types` regenerates `src/bindings.rs` and
  `test/src/env.rs` after editing the `.d.ts` files.
- `chomp test:ts-gen-worker` builds the test worker and runs its vitest suite
  under miniflare.
