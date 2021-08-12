# workers-rs

**Work-in-progress** ergonomic Rust bindings to Cloudflare Workers environment. Write your entire worker in Rust!

## Project Contents

- **edgeworker-ffi**: Rust extern "C" definitions for FFI compatibility with the Workers JS Runtime.
- **libworker**: wrappers and convenience library over the FFI bindings.
- **macros**: `cf` exports macros for wrapping Rust entry point in a `fetch` method of an ES Module, and code generation to create and interact with Durable Objects.
- **worker**: the user-facing crate, with Rust-famaliar abstractions over the Rust<->JS/WebAssembly interop.
- **rust-sandbox**: a functioning Cloudflare Worker for testing features and ergonomics.

## Example Usage

```rust
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, _env: Env) -> Result<Response> {
    console_log!("request at: {:?}", req.path());

    utils::set_panic_hook();

    let mut router = Router::new();

    router.on("/user/:id/test", |req, _env, params| {
        if !matches!(req.method(), Method::Get) {
            return Response::error("Method Not Allowed".into(), 405);
        }
        let id = params.get("id").unwrap_or("not found");
        Response::ok(format!("TEST user id: {}", id))
    })?;

    router.post("/headers", |req, _env, _params| {
        let mut headers: http::HeaderMap = req.headers().into();
        headers.append("Hello", "World!".parse().unwrap());

        Response::ok("returned your headers to you.".into())
            .map(|res| res.with_headers(headers.into()))
    })?;
    
    router.on_async("/fetch_json", |_req, _env, _params| async move {
        let data: Todo = Fetch::Url("https://jsonplaceholder.typicode.com/todos/1")
            .send()
            .await?
            .json()
            .await?;

        Response::ok(format!(
            "API Returned user: {} with title: {} and completed: {}",
            data.user_id, data.title, data.completed
        ))
    })?;

    router.get_async("/proxy_request/:url", |_req, _env, params| {
        // Must copy the parameters into the heap here for lifetime purposes
        let url = params.get("url").unwrap().to_string();
        async move { Fetch::Url(&url).send().await }
    })?;

    router.get("/secret", |_req, env, _params| {
        Response::ok(env.secret("SOME_SECRET")?.to_string())
    })?;

    router.get("/var", |_req, env, _params| {
        Response::ok(env.var("SOME_VARIABLE")?.to_string())
    })?;

    router.post_async("/kv", |_req, env, _params| async move {
        let kv = env.kv("SOME_NAMESPACE")?;
        kv.put("another-key", "another-value")?.execute().await?;

        Response::empty()
    })?;

    router.run(req).await
```

## Getting Started

Make sure you have [`wrangler`](https://github.com/cloudflare/wrangler) installed at a recent version (>v1.19.0). If you want to publish your Rust worker code, you will need to have a [Cloudflare account](https://cloudflare.com).

Run `wrangler --version` to check your installation and if it meets the version requirements.

```bash
wrangler generate --type=rust project_name
cd project_name
wrangler build
```

You should see a new project layout with a `src/lib.rs`. Start there! Use any local or remote crates and modules (as long as they compile to the `wasm32-unknown-unknown` target). 

Once you're ready to run your project:

```bash
wrangler dev
```

And then go live:
```bash
# configure your routes, zones & more in your worker's `wrangler.toml` file
wrangler login
```

## KV, Secret, & Variable Bindings

All "bindings" to your script (KV Namespaces, Secrets, and Variables) are accessible from the `env` parameter provided to both the entrypoint (`main` in this example), and to the route handler closure, if you use the `Router` from the `worker` crate.

```rust
use worker::*;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, _env: Env) -> Result<Response> {
    utils::set_panic_hook();

    let mut router = Router::new();

    router.get("/secret", |_req, env, _params| {
        Response::ok(env.secret("SOME_SECRET")?.to_string())
    })?;

    router.get("/var", |_req, env, _params| {
        Response::ok(env.var("SOME_VARIABLE")?.to_string())
    })?;

    router.post_async("/kv", |_req, env, _params| async move {
        let kv = env.kv("SOME_NAMESPACE")?;
        kv.put("another-key", "another-value")?.execute().await?;

        Response::empty()
    })?;

    router.run(req, env).await
}
```

## Durable Objects

### BETA WARNING
Durable Objects are still in **BETA**, so the same rules apply to the Durable Object code and APIs here in these crates.

### Define a Durable Object in Rust
To define a Durable Object using the `worker` crate you need to implement the `DurableObject` trait on your own struct. Additionally, the `#[durable_object]` attribute macro must be applied to _both_ your struct definition and the trait `impl` block for it.

```rust
use worker::*;

#[durable_object]
pub struct Chatroom {
    users: Vec<User>,
    messages: Vec<Message>
    state: State,
    some_secret: Secret,
    some_kv_store: KvStore,

}

#[durable_object]
impl DurableObject for Chatroom {
    fn constructor(state: State, _env: Env) -> Self {
        Self {
            users: vec![],
            messages: vec![],
            state: state,
            some_secret: env.secret("SOME_SECRET").unwrap(),
            some_kv_store: env.kv("SOME_KV_STORE").unwrap()
        }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // do some work when a worker makes a request to this DO
        Response::ok(&format!("{} active users.", self.users.len()))
    }
}
```

You'll need to "migrate" your worker script when it's published so that it is aware of this new Durable Object, and include a binding in your `wrangler.toml`.

- Include the Durable Object binding type in you `wrangler.toml` file:

```toml
# ...

[durable_objects]
bindings = [
  { name = "CHATROOM", class_name = "Chatroom" } # the `class_name` uses the Rust struct identifier name
]

[[migrations]]
tag = "v1" # Should be unique for each entry
new_classes = ["Chatroom"] # Array of new classes
```

- For more information about migrating your Durable Object as it changes, see the docs here: https://developers.cloudflare.com/workers/learning/using-durable-objects#durable-object-migrations-in-wranglertoml

