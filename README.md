# workers-rs

**Work-in-progress** ergonomic Rust bindings to Cloudflare Workers environment. Write your entire worker in Rust!

```rust
#[cf::worker(fetch)]
pub async fn main(req: Request) -> Result<Response> {
    console_log!("request at: {:?}", req.path());

    utils::set_panic_hook();

    let mut router = Router::new();

    router.post("/headers", |req, _| {
        let mut headers: http::HeaderMap = req.headers().into();
        headers.append("Hello", "World!".parse().unwrap());

        Response::ok("returned your headers to you.".into())
            .map(|res| res.with_headers(headers.into()))
    })?;

    router.on("/user/:id/test", |req, params| {
        if !matches!(req.method(), Method::Get) {
            return Response::error("Method Not Allowed".into(), 405);
        }
        let id = params.get("id").unwrap_or("not found");
        Response::ok(format!("TEST user id: {}", id))
    })?;


    router.post("/account/:id/zones", |_, params| {
        Response::ok(format!(
            "Create new zone for Account: {}",
            params.get("id").unwrap_or("not found")
        ))
    })?;

    router.get("/account/:id/zones", |_, params| {
        Response::ok(format!(
            "Account id: {}..... You get a zone, you get a zone!",
            params.get("id").unwrap_or("not found")
        ))
    })?;

    router.on_async("/fetch", |_req, _params| async move {
        let req = Request::new("https://example.com", "POST")?;
        let resp = Fetch::Request(&req).send().await?;
        let resp2 = Fetch::Url("https://example.com").send().await?;
        Response::ok(format!(
            "received responses with codes {} and {}",
            resp.status_code(),
            resp2.status_code()
        ))
    })?;
    
    router.on_async("/fetch_json", |_req, _params| async move {
        let data: ApiData = Fetch::Url("https://jsonplaceholder.typicode.com/todos/1")
            .send()
            .await?
            .json()
            .await?;
        Response::ok(format!(
            "API Returned user: {} with title: {} and completed: {}",
            data.user_id, data.title, data.completed
        ))
    })?;

    router.on_async("/proxy_request/:url", |_req, params| {
        // Must copy the parameters into the heap here for lifetime purposes
        let url = params.get("url").unwrap().to_string();
        async move { Fetch::Url(&url).send().await }
    })?;

    router.run(req).await
```

## repo layout

- edgeworker-sys = same as web-sys, and even some copy/pasted externs. these need to be slimmed down to only be what worker runtime supports, and added to with stuff I haven't got to
- macros = cf macro is the one that hoists your code into a "glue" conversion thing.. not super important, but makes it automatically nicer to work with
- worker = the convenience wrapper types & fn's on top of edgeworker-sys
- rust-sandbox = the example worker I use to play with all this stuff
