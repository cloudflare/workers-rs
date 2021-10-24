use blake2::{Blake2b, Digest};
use chrono;
use cloudflare::framework::{async_api::Client, Environment, HttpApiClientConfig};
use serde::{Deserialize, Serialize};
use worker::*;

mod counter;
mod test;
mod utils;

#[derive(Deserialize, Serialize)]
struct MyData {
    message: String,
    #[serde(default)]
    is: bool,
    #[serde(default)]
    data: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiData {
    user_id: i32,
    title: String,
    completed: bool,
}

#[derive(Serialize)]
struct User {
    id: String,
    timestamp: u64,
    date_from_int: String,
    date_from_str: String,
}

#[derive(Deserialize, Serialize)]
struct FileSize {
    name: String,
    size: u32,
}

struct SomeSharedData {
    regex: regex::Regex,
    cloudflare_api_client: Client,
}

fn handle_a_request<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(&format!(
        "req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    ))
}

async fn handle_async_request<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(&format!(
        "[async] req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    ))
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    utils::set_panic_hook();

    let creds = cloudflare::framework::auth::Credentials::UserAuthToken {
        token: env.secret("CF_API_TOKEN")?.to_string(),
    };
    let mut config = HttpApiClientConfig::default();
    config.http_timeout = std::time::Duration::from_millis(500);
    let client = Client::new(creds, config, Environment::Production).unwrap();

    let data = SomeSharedData {
        regex: regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
        cloudflare_api_client: client,
    };

    let router = Router::with_data(data); // if no data is needed, pass `()` or any other valid data

    router
        .get("/request", handle_a_request) // can pass a fn pointer to keep routes tidy
        .get_async("/async-request", handle_async_request)
        .get("/test-data", |_, ctx| {
            // just here to test data works
            if ctx.data().regex.is_match("2014-01-01") {
                Response::ok("data ok")
            } else {
                Response::error("bad match", 500)
            }
        })
        .post("/headers", |req, _ctx| {
            let mut headers: http::HeaderMap = req.headers().into();
            headers.append("Hello", "World!".parse().unwrap());

            Response::ok("returned your headers to you.")
                .map(|res| res.with_headers(headers.into()))
        })
        .post_async("/formdata-name", |mut req, _ctx| async move {
            let form = req.form_data().await?;
            const NAME: &str = "name";
            let bad_request = Response::error("Bad Request", 400);

            if !form.has(NAME) {
                return bad_request;
            }

            let names: Vec<String> = form
                .get_all(NAME)
                .unwrap_or_default()
                .into_iter()
                .map(|entry| match entry {
                    FormEntry::Field(s) => s,
                    FormEntry::File(f) => f.name(),
                })
                .collect();
            if names.len() > 1 {
                return Response::from_json(&serde_json::json!({ "names": names }));
            }

            if let Some(value) = form.get(NAME) {
                match value {
                    FormEntry::Field(v) => Response::from_json(&serde_json::json!({ NAME: v })),
                    _ => bad_request,
                }
            } else {
                bad_request
            }
        })
        .post_async("/is-secret", |mut req, ctx| async move {
            let form = req.form_data().await?;
            if let Some(secret) = form.get("secret") {
                match secret {
                    FormEntry::Field(name) => {
                        let val = ctx.secret(&name)?;
                        return Response::ok(val.to_string());
                    }
                    _ => return Response::error("Bad Request", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .post_async("/formdata-file-size", |mut req, ctx| async move {
            let form = req.form_data().await?;

            if let Some(entry) = form.get("file") {
                return match entry {
                    FormEntry::File(file) => {
                        let kv = ctx.kv("FILE_SIZES")?;

                        // create a new FileSize record to store
                        let b = file.bytes().await?;
                        let record = FileSize {
                            name: file.name(),
                            size: b.len() as u32,
                        };

                        // hash the file, and use result as the key
                        let mut hasher = Blake2b::new();
                        hasher.update(b);
                        let hash = hasher.finalize();
                        let key = hex::encode(&hash[..]);

                        // serialize the record and put it into kv
                        let val = serde_json::to_string(&record)?;
                        kv.put(&key, val)?.execute().await?;

                        // list the default number of keys from the namespace
                        Response::from_json(&kv.list().execute().await?.keys)
                    }
                    _ => Response::error("Bad Request", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .get_async("/formdata-file-size/:hash", |_, ctx| async move {
            if let Some(hash) = ctx.param("hash") {
                let kv = ctx.kv("FILE_SIZES")?;
                return match kv.get(&hash).await? {
                    Some(val) => Response::from_json(&val.as_json::<FileSize>()?),
                    None => Response::error("Not Found", 404),
                };
            }

            Response::error("Bad Request", 400)
        })
        .post_async("/post-file-size", |mut req, _| async move {
            let bytes = req.bytes().await?;
            Response::ok(&format!("size = {}", bytes.len()))
        })
        .get("/user/:id/test", |_req, ctx| {
            if let Some(id) = ctx.param("id") {
                return Response::ok(format!("TEST user id: {}", id));
            }

            Response::error("Error", 500)
        })
        .get("/user/:id", |_req, ctx| {
            if let Some(id) = ctx.param("id") {
                return Response::from_json(&User {
                    id: id.to_string(),
                    timestamp: Date::now().as_millis(),
                    date_from_int: Date::new(DateInit::Millis(1234567890)).to_string(),
                    date_from_str: Date::new(DateInit::String(
                        "Wed Jan 14 1980 23:56:07 GMT-0700 (Mountain Standard Time)".into(),
                    ))
                    .to_string(),
                });
            }

            Response::error("Bad Request", 400)
        })
        .post("/account/:id/zones", |_, ctx| {
            Response::ok(format!(
                "Create new zone for Account: {}",
                ctx.param("id").unwrap_or(&"not found".into())
            ))
        })
        .get("/account/:id/zones", |_, ctx| {
            Response::ok(format!(
                "Account id: {}..... You get a zone, you get a zone!",
                ctx.param("id").unwrap_or(&"not found".into())
            ))
        })
        .get_async("/async-text-echo", |mut req, _ctx| async move {
            Response::ok(req.text().await?)
        })
        .get_async("/fetch", |_req, _ctx| async move {
            let req = Request::new("https://example.com", Method::Post)?;
            let resp = Fetch::Request(req).send().await?;
            let resp2 = Fetch::Url("https://example.com".parse()?).send().await?;
            Response::ok(format!(
                "received responses with codes {} and {}",
                resp.status_code(),
                resp2.status_code()
            ))
        })
        .get_async("/fetch_json", |_req, _ctx| async move {
            let data: ApiData = Fetch::Url(
                "https://jsonplaceholder.typicode.com/todos/1"
                    .parse()
                    .unwrap(),
            )
            .send()
            .await?
            .json()
            .await?;
            Response::ok(format!(
                "API Returned user: {} with title: {} and completed: {}",
                data.user_id, data.title, data.completed
            ))
        })
        .get_async("/cloudflare-api", |_req, ctx| async move {
            let resp = ctx
                .data()
                .cloudflare_api_client
                .request_handle(&cloudflare::endpoints::user::GetUserDetails {})
                .await
                .unwrap();

            Response::ok("hello user").map(|res| {
                let mut headers = Headers::new();
                headers
                    .set("user-details-email", &resp.result.email)
                    .unwrap();
                res.with_headers(headers)
            })
        })
        .get_async("/proxy_request/*url", |_req, ctx| async move {
            let url = ctx.param("url").unwrap().strip_prefix('/').unwrap();

            Fetch::Url(url.parse()?).send().await
        })
        .get_async("/durable/:id", |_req, ctx| async move {
            let namespace = ctx.durable_object("COUNTER")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            stub.fetch_with_str("/").await
        })
        .get("/secret", |_req, ctx| {
            Response::ok(ctx.secret("SOME_SECRET")?.to_string())
        })
        .get("/var", |_req, ctx| {
            Response::ok(ctx.var("SOME_VARIABLE")?.to_string())
        })
        .post_async("/kv/:key/:value", |_req, ctx| async move {
            let kv = ctx.kv("SOME_NAMESPACE")?;
            if let Some(key) = ctx.param("key") {
                if let Some(value) = ctx.param("value") {
                    kv.put(&key, value)?.execute().await?;
                }
            }

            Response::from_json(&kv.list().execute().await?)
        })
        .get("/bytes", |_, _| {
            Response::from_bytes(vec![1, 2, 3, 4, 5, 6, 7])
        })
        .post_async("/api-data", |mut req, _ctx| async move {
            let data = req.bytes().await?;
            let mut todo: ApiData = serde_json::from_slice(&data)?;

            unsafe { todo.title.as_mut_vec().reverse() };

            console_log!("todo = (title {}) (id {})", todo.title, todo.user_id);

            Response::from_bytes(serde_json::to_vec(&todo)?)
        })
        .post_async("/nonsense-repeat", |_, ctx| async move {
            //  just here to test data works, and verify borrow
            let _d = ctx.data();

            if ctx.data().regex.is_match("2014-01-01") {
                Response::ok("data ok")
            } else {
                Response::error("bad match", 500)
            }
        })
        .get("/status/:code", |_, ctx| {
            if let Some(code) = ctx.param("code") {
                return match code.parse::<u16>() {
                    Ok(status) => Response::ok("You set the status code!")
                        .map(|resp| resp.with_status(status)),
                    Err(_e) => Response::error("Failed to parse your status code.", 400),
                };
            }

            Response::error("Bad Request", 400)
        })
        .put("/", respond)
        .patch("/", respond)
        .delete("/", respond)
        .head("/", respond)
        .put_async("/async", respond_async)
        .patch_async("/async", respond_async)
        .delete_async("/async", respond_async)
        .head_async("/async", respond_async)
        .options("/*catchall", |_, ctx| {
            Response::ok(ctx.param("catchall").unwrap())
        })
        .get_async("/request-init-fetch", |_, _| async move {
            let init = RequestInit::new();
            Fetch::Request(Request::new_with_init("https://cloudflare.com", &init)?)
                .send()
                .await
        })
        .get_async("/request-init-fetch-post", |_, _| async move {
            let mut init = RequestInit::new();
            init.method = Method::Post;
            Fetch::Request(Request::new_with_init("https://httpbin.org/post", &init)?)
                .send()
                .await
        })
        .get("/redirect-default", |_, _| {
            Response::redirect("https://example.com".parse().unwrap())
        })
        .get("/redirect-307", |_, _| {
            Response::redirect_with_status("https://example.com".parse().unwrap(), 307)
        })
        .get("/now", |_, _| {
            let now = chrono::Utc::now();
            let js_date: Date = now.date().into();
            Response::ok(format!("{}", js_date.to_string()))
        })
        .or_else_any_method_async("/*catchall", |_, ctx| async move {
            console_log!(
                "[or_else_any_method_async] caught: {}",
                ctx.param("catchall").unwrap_or(&"?".to_string())
            );

            Fetch::Url("https://github.com/404".parse().unwrap())
                .send()
                .await
                .map(|resp| resp.with_status(404))
        })
        .run(req, env)
        .await
}

fn respond<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!("Ok: {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}

async fn respond_async<D>(req: Request, _ctx: RouteContext<D>) -> Result<Response> {
    Response::ok(format!("Ok (async): {}", String::from(req.method()))).map(|resp| {
        let mut headers = Headers::new();
        headers.set("x-testing", "123").unwrap();
        resp.with_headers(headers)
    })
}
