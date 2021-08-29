# workers-rs

**Work-in-progress** ergonomic Rust bindings to Cloudflare Workers environment. Write your entire worker in Rust!

## Example Usage

```rust
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    console_log!(
        "{} {}, located at: {:?}, within: {}",
        req.method().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );

    if !matches!(req.method(), Method::Post) {
        return Response::error("Method Not Allowed", 405);
    }

    if let Some(file) = req.form_data().await?.get("file") {
        return match file {
            FormEntry::File(buf) => {
                Response::ok(&format!("size = {}", buf.bytes().await?.len()))
            }
            _ => Response::error("`file` part of POST form must be a file", 400),
        };
    }

    Response::error("Bad Request", 400)
}
```

### Or use the `router::Router`:

Parameterize routes and access the parameter values from within a handler. Each hanlder function takes a 
`Request`, an `Env`, and a `RouteParams` (`HashMap<String, String>`). 

```rust
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    let mut router = Router::new();

    // useful for JSON APIs
    #[derive(Deserialize, Serialize)]
    struct Account {
        id: u64,
        // ...
    }
    router.get_async("/account/:id", |_req, env, params| async move {
        if let Some(id) = params.get("id") {
            let accounts = env.kv("ACCOUNTS")?;
            return match accounts.get(id).await? {
                Some(account) => Response::from_json(&account.as_json::<Account>()?),
                None => Response::error("Not found", 404),
            };
        }

        Response::error("Bad Request", 400)
    })?;

    // handle files and fields from multipart/form-data requests
    router.post_async("/upload", |mut req, _, _| async move {
        let form = req.form_data().await?;
        if let Some(entry) = form.get("file") {
            match entry {
                FormEntry::File(file) => {
                    let bytes = file.bytes().await?;
                }
                FormEntry::Field(_) => return Response::error("Bad Request", 400),
            }
            // ...

            if let Some(permissions) = form.get("permissions") {
                // permissions == "a,b,c,d"
            }
            // or call `form.get_all("permissions")` if using multiple entries per field
        }

        Response::error("Bad Request", 400)
    })?;

    // read/write binary data
    router.post_async("/echo-bytes", |mut req, _, _| async move {
        let data = req.bytes().await?;
        if data.len() < 1024 {
            return Response::error("Bad Request", 400);
        }

        Response::from_bytes(data)
    })?;

    router.run(req, env).await
}   
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
wrangler publish
```

## Durable Object, KV, Secret, & Variable Bindings

All "bindings" to your script (Durable Object & KV Namespaces, Secrets, and Variables) are accessible from the `env` parameter provided to both the entrypoint (`main` in this example), and to the route handler callback, if you use the `Router` from the `worker` crate.

```rust
use worker::*;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, _env: Env) -> Result<Response> {
    utils::set_panic_hook();

    let mut router = Router::new();

    router.on_async("/durable", |_req, env, _params| async move {
        let namespace = env.durable_object("CHATROOM")?;
        let stub = namespace.id_from_name("A")?.get_stub()?;
        stub.fetch_with_str("/messages").await
    })?;

    router.get("/secret", |_req, env, _params| {
        Response::ok(env.secret("CF_API_TOKEN")?.to_string())
    })?;

    router.get("/var", |_req, env, _params| {
        Response::ok(env.var("BUILD_NUMBER")?.to_string())
    })?;

    router.post_async("/kv", |req, env, _params| async move {
        let kv = env.kv("SOME_NAMESPACE")?;

        kv.put("key", "value")?.execute().await?;

        Response::empty()
    })?;

    router.run(req, env).await
}
```

For more information about how to configure these bindings, see: 
- https://developers.cloudflare.com/workers/cli-wrangler/configuration#keys
- https://developers.cloudflare.com/workers/learning/using-durable-objects#configuring-durable-object-bindings

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
    env: std::sync::Arc<Env>, // access `Env` across requests, use inside `fetch`

}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        Self {
            users: vec![],
            messages: vec![],
            state: state,
            env: std::sync::Arc::new(env),
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


# Contributing

Your feedback is welcome and appreciated! Please use the issue tracker to talk about potential implementations or make feature requests. If you're interested in making a PR, we suggest opening up an issue to talk about the change you'd like to make as early as possible.

## Project Contents

- **worker**: the user-facing crate, with Rust-famaliar abstractions over the Rust<->JS/WebAssembly interop.
- **libworker**: wrappers and convenience library over the FFI bindings.
- **edgeworker-sys**: Rust extern "C" definitions for FFI compatibility with the Workers JS Runtime.
- **macros**: exports `event` and `durable_object` macros for wrapping Rust entry point in a `fetch` method of an ES Module, and code generation to create and interact with Durable Objects.
- **rust-sandbox**: a functioning Cloudflare Worker for testing features and ergonomics.
- **rust-worker-build**: a cross-platform build command for `workers-rs`-based projects.