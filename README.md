![workers-rs](.github/logo.png)
[![crates.io](https://img.shields.io/crates/v/worker)](https://crates.io/crates/worker)
[![docs.rs](https://img.shields.io/docsrs/worker)](https://docs.rs/worker)

**Work-in-progress** ergonomic Rust bindings to Cloudflare Workers environment. Write your entire worker in Rust!

Read the [Notes and FAQ](#notes-and-faq)

## Example Usage

```rust
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
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

### Or use the `Router`:

Parameterize routes and access the parameter values from within a handler. Each handler function takes a
`Request`, and a `RouteContext`. The `RouteContext` has shared data, route params, `Env` bindings, and more.

```rust
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {

    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    // useful for JSON APIs
    #[derive(Deserialize, Serialize)]
    struct Account {
        id: u64,
        // ...
    }
    router
        .get_async("/account/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                let accounts = ctx.kv("ACCOUNTS")?;
                return match accounts.get(id).json::<Account>().await? {
                    Some(account) => Response::from_json(&account),
                    None => Response::error("Not found", 404),
                };
            }

            Response::error("Bad Request", 400)
        })
        // handle files and fields from multipart/form-data requests
        .post_async("/upload", |mut req, _ctx| async move {
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
        })
        // read/write binary data
        .post_async("/echo-bytes", |mut req, _ctx| async move {
            let data = req.bytes().await?;
            if data.len() < 1024 {
                return Response::error("Bad Request", 400);
            }

            Response::from_bytes(data)
        })
        .run(req, env).await
}
```

## Getting Started

The project uses [wrangler](https://github.com/cloudflare/wrangler2) version 2.x for running and publishing your Worker.

Get the Rust worker project [template](https://github.com/cloudflare/templates/tree/main/worker-rust) manually, or run the following command:
```bash
npm init cloudflare project_name worker-rust
cd project_name
```

You should see a new project layout with a `src/lib.rs`. Start there! Use any local or remote crates
and modules (as long as they compile to the `wasm32-unknown-unknown` target).

Once you're ready to run your project:

First check that the wrangler version is 2.x
```bash
npx wrangler --version
```

Then, run your worker

```bash
npx wrangler dev
```

Finally, go live:

```bash
# configure your routes, zones & more in your worker's `wrangler.toml` file
npx wrangler publish
```

If you would like to have `wrangler` installed on your machine, see instructions in [wrangler repository](https://github.com/cloudflare/wrangler2).
## Durable Object, KV, Secret, & Variable Bindings

All "bindings" to your script (Durable Object & KV Namespaces, Secrets, and Variables) are
accessible from the `env` parameter provided to both the entrypoint (`main` in this example), and to
the route handler callback (in the `ctx` argument), if you use the `Router` from the `worker` crate.

```rust
use worker::*;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();

    router
        .on_async("/durable", |_req, ctx| async move {
            let namespace = ctx.durable_object("CHATROOM")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            stub.fetch_with_str("/messages").await
        })
        .get("/secret", |_req, ctx| {
            Response::ok(ctx.secret("CF_API_TOKEN")?.to_string())
        })
        .get("/var", |_req, ctx| {
            Response::ok(ctx.var("BUILD_NUMBER")?.to_string())
        })
        .post_async("/kv", |_req, ctx| async move {
            let kv = ctx.kv("SOME_NAMESPACE")?;

            kv.put("key", "value")?.execute().await?;

            Response::empty()
        })
        .run(req, env).await
}
```

For more information about how to configure these bindings, see:

- https://developers.cloudflare.com/workers/cli-wrangler/configuration#keys
- https://developers.cloudflare.com/workers/learning/using-durable-objects#configuring-durable-object-bindings

## Durable Objects

### Define a Durable Object in Rust

To define a Durable Object using the `worker` crate you need to implement the `DurableObject` trait
on your own struct. Additionally, the `#[durable_object]` attribute macro must be applied to _both_
your struct definition and the trait `impl` block for it.

```rust
use worker::*;

#[durable_object]
pub struct Chatroom {
    users: Vec<User>,
    messages: Vec<Message>,
    state: State,
    env: Env, // access `Env` across requests, use inside `fetch`
}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        Self {
            users: vec![],
            messages: vec![],
            state: state,
            env,
        }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // do some work when a worker makes a request to this DO
        Response::ok(&format!("{} active users.", self.users.len()))
    }
}
```

You'll need to "migrate" your worker script when it's published so that it is aware of this new
Durable Object, and include a binding in your `wrangler.toml`.

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

- For more information about migrating your Durable Object as it changes, see the docs here:
  https://developers.cloudflare.com/workers/learning/using-durable-objects#durable-object-migrations-in-wranglertoml

## Queues

### Enabling queues
As queues are in beta you need to enable the `queue` feature flag.

Enable it by adding it to the worker dependency in your `Cargo.toml`: 
```toml
worker = {version = "...", features = ["queue"]}
```

### Example worker consuming and producing messages:
```rust
use worker::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct MyType {
    foo: String,
    bar: u32,
}

// Consume messages from a queue
#[event(queue)]
pub async fn main(message_batch: MessageBatch<MyType>, env: Env, _ctx: Context) -> Result<()> {
    // Get a queue with the binding 'my_queue'
    let my_queue = env.queue("my_queue")?;

    // Deserialize the message batch
    let messages = message_batch.messages()?;

    // Loop through the messages
    for message in messages {
        // Log the message and meta data
        console_log!(
            "Got message {:?}, with id {} and timestamp: {}",
            message.body,
            message.id,
            message.timestamp.to_string()
        );

        // Send the message body to the other queue
        my_queue.send(&message.body).await?;
    }

    // Retry all messages
    message_batch.retry_all();
    Ok(())
}
```

## D1 Databases

### Enabling D1 databases
As D1 databases are in alpha, you'll need to enable the `d1` feature on the `worker` crate.

```toml
worker = { version = "x.y.z", features = ["d1"] }
```

### Example usage
```rust
use worker::*;

#[derive(Deserialize)]
struct Thing {
	thing_id: String,
	desc: String,
	num: u32,
}

#[event(fetch, respond_with_errors)]
pub async fn main(request: Request,	env: Env, _ctx: Context) -> Result<Response> {
	Router::new()
		.get_async("/:id", |_, ctx| async move {
			let id = ctx.param("id").unwrap()?;
			let d1 = ctx.env.d1("things-db")?;
			let statement = d1.prepare("SELECT * FROM things WHERE thing_id = ?1");
			let query = statement.bind(&[id])?;
			let result = query.first::<Thing>(None).await?;
			match result {
				Some(thing) => Response::from_json(&thing),
				None => Response::error("Not found", 404),
			}
		})
		.run(request, env)
		.await
}
```


# Notes and FAQ

It is exciting to see how much is possible with a framework like this, by expanding the options
developers have when building on top of the Workers platform. However, there is still much to be
done. Expect a few rough edges, some unimplemented APIs, and maybe a bug or two here and there. It’s
worth calling out here that some things that may have worked in your Rust code might not work here -
it’s all WebAssembly at the end of the day, and if your code or third-party libraries don’t target
`wasm32-unknown-unknown`, they can’t be used on Workers. Additionally, you’ve got to leave your
threaded async runtimes at home; meaning no Tokio or async_std support. However, async/await syntax
is still available and supported out of the box when you use the `worker` crate.

We fully intend to support this crate and continue to build out its missing features, but your help
and feedback is a must. We don’t like to build in a vacuum, and we’re in an incredibly fortunate
position to have brilliant customers like you who can help steer us towards an even better product.

So give it a try, leave some feedback, and star the repo to encourage us to dedicate more time and
resources to this kind of project.

If this is interesting to you and you want to help out, we’d be happy to get outside contributors
started. We know there are improvements to be made such as compatibility with popular Rust HTTP
ecosystem types (we have an example conversion for [Headers](https://github.com/cloudflare/workers-rs/blob/3d5876a1aca0a649209152d1ffd52dae7bccda87/libworker/src/headers.rs#L131-L167) if you want to make one), implementing additional Web APIs, utility crates,
and more. In fact, we’re always on the lookout for great engineers, and hiring for many open roles -
please [take a look](https://www.cloudflare.com/careers/).

### FAQ

1. Can I deploy a Worker that uses `tokio` or `async_std` runtimes?

- Currently no. All crates in your Worker project must compile to `wasm32-unknown-unknown` target,
  which is more limited in some ways than targets for x86 and ARM64.

2. The `worker` crate doesn't have _X_! Why not?

- Most likely, it should, we just haven't had the time to fully implement it or add a library to
  wrap the FFI. Please let us know you need a feature by [opening an issue](https://github.com/cloudflare/workers-rs/issues).

3. My bundle size exceeds Workers 1MB limits, what do I do?

- We're working on solutions here, but in the meantime you'll need to minimize the number of crates
  your code depends on, or strip as much from the `.wasm` binary as possible. Here are some extra
  steps you can try: https://rustwasm.github.io/book/reference/code-size.html#optimizing-builds-for-code-size

# Contributing

Your feedback is welcome and appreciated! Please use the issue tracker to talk about potential
implementations or make feature requests. If you're interested in making a PR, we suggest opening up
an issue to talk about the change you'd like to make as early as possible.

## Project Contents

- **worker**: the user-facing crate, with Rust-familiar abstractions over the Rust<->JS/WebAssembly
  interop via wrappers and convenience library over the FFI bindings.
- **worker-sys**: Rust extern "C" definitions for FFI compatibility with the Workers JS Runtime.
- **worker-macros**: exports `event` and `durable_object` macros for wrapping Rust entry point in a
  `fetch` method of an ES Module, and code generation to create and interact with Durable Objects.
- **worker-sandbox**: a functioning Cloudflare Worker for testing features and ergonomics.
- **worker-build**: a cross-platform build command for `workers-rs`-based projects.
