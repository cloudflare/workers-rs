use crate::SomeSharedData;

use super::ApiData;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use worker::Env;
use worker::{
    console_log, ok, Date, Delay, Request, Response, ResponseBody, ResponseBuilder, Result,
};

#[allow(clippy::needless_pass_by_value)]
pub fn handle_a_request(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(format!(
        "req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().map(|cf| cf.coordinates().unwrap_or_default()),
        req.cf().map_or(String::from("No CF properties"), |cf| cf
            .region()
            .unwrap_or_else(|| "unknown region".into()))
    ))
}

pub async fn handle_async_request(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::ok(format!(
        "[async] req at: {}, located at: {:?}, within: {}",
        req.path(),
        req.cf().map(|cf| cf.coordinates().unwrap_or_default()),
        req.cf().map_or(String::from("No CF properties"), |cf| cf
            .region()
            .unwrap_or_else(|| "unknown region".into()))
    ))
}

pub async fn handle_test_data(_req: Request, _env: Env, data: SomeSharedData) -> Result<Response> {
    // just here to test data works
    if data.regex.is_match("2014-01-01") {
        Response::ok("data ok")
    } else {
        Response::error("bad match", 500)
    }
}

pub async fn handle_xor(mut req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = req.url()?;
    let num: u8 = match url.path_segments().unwrap().nth(1).unwrap().parse() {
        Ok(num) => num,
        Err(_) => return Response::error("invalid byte", 400),
    };

    let xor_stream = req.stream()?.map_ok(move |mut buf| {
        buf.iter_mut().for_each(|x| *x ^= num);
        buf
    });

    Response::from_stream(xor_stream)
}

pub async fn handle_headers(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let mut headers: http::HeaderMap = req.headers().into();
    headers.append("Hello", "World!".parse().unwrap());

    ResponseBuilder::new()
        .with_headers(headers.into())
        .ok("returned your headers to you.")
}

pub async fn handle_post_file_size(
    mut req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let bytes = req.bytes().await?;
    Response::ok(format!("size = {}", bytes.len()))
}

pub async fn handle_async_text_echo(
    mut req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::ok(req.text().await?)
}

pub async fn handle_secret(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(env.secret("SOME_SECRET")?.to_string())
}

pub async fn handle_var(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::ok(env.var("SOME_VARIABLE")?.to_string())
}

pub async fn handle_object_var(_req: Request, env: Env, _: SomeSharedData) -> Result<Response> {
    #[derive(Serialize, Deserialize, PartialEq, Eq)]
    struct Obj {
        foo: i32,
        bar: String,
    }
    let obj = env.object_var::<Obj>("SOME_OBJECT_VARIABLE")?;
    Response::from_json(&obj)
}

pub async fn handle_bytes(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    Response::from_bytes(vec![1, 2, 3, 4, 5, 6, 7])
}

pub async fn handle_api_data(
    mut req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let data = req.bytes().await?;
    let mut todo: ApiData = match serde_json::from_slice(&data) {
        Ok(todo) => todo,
        Err(e) => {
            return Response::ok(e.to_string());
        }
    };

    unsafe { todo.title.as_mut_vec().reverse() };

    console_log!("todo = (title {}) (id {})", todo.title, todo.user_id);

    Response::from_bytes(serde_json::to_vec(&todo)?)
}

pub async fn handle_nonsense_repeat(
    _req: Request,
    _env: Env,
    data: SomeSharedData,
) -> Result<Response> {
    if data.regex.is_match("2014-01-01") {
        Response::ok("data ok")
    } else {
        Response::error("bad match", 500)
    }
}

pub async fn handle_status(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    let code = segments.nth(1);
    if let Some(code) = code {
        return match code.parse::<u16>() {
            Ok(status) => ResponseBuilder::new()
                .with_status(status)
                .ok("You set the status code!"),
            Err(_e) => Response::error("Failed to parse your status code.", 400),
        };
    }

    Response::error("Bad Request", 400)
}

pub async fn handle_redirect_default(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::redirect("https://example.com".parse().unwrap())
}

pub async fn handle_redirect_307(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::redirect_with_status("https://example.com".parse().unwrap(), 307)
}

pub async fn handle_now(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let now = chrono::Utc::now();
    let js_date: Date = now.into();
    Response::ok(js_date.to_string())
}

pub async fn handle_cloned(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let mut resp = Response::ok("Hello")?;
    let mut resp1 = resp.cloned()?;

    let left = resp.text().await?;
    let right = resp1.text().await?;

    Response::ok((left == right).to_string())
}

pub async fn handle_cloned_stream(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let stream =
        futures_util::stream::repeat(())
            .take(10)
            .enumerate()
            .then(|(index, ())| async move {
                Delay::from(Duration::from_millis(100)).await;
                ok::Ok(index.to_string().into_bytes())
            });

    let mut resp = Response::from_stream(stream)?;
    let mut resp1 = resp.cloned()?;

    let left = resp.text().await?;
    let right = resp1.text().await?;

    Response::ok((left == right).to_string())
}

pub async fn handle_custom_response_body(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Response::from_body(ResponseBody::Body(vec![b'h', b'e', b'l', b'l', b'o']))
}

pub async fn handle_wait_delay(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = req.url()?;
    let mut segments = uri.path_segments().unwrap();
    let delay = segments.nth(1);
    let delay: Delay = match delay.unwrap().parse() {
        Ok(delay) => Duration::from_millis(delay).into(),
        Err(_) => return Response::error("invalid delay", 400),
    };

    // Wait for the delay to pass
    delay.await;

    Response::ok("Waited!\n")
}

// Compile-time assertion: public async Request methods return Send futures.
#[allow(dead_code, unused)]
fn _assert_send() {
    fn require_send<T: Send>(_t: T) {}
    fn request(mut req: worker::Request) {
        require_send(req.json::<String>());
        require_send(req.text());
        require_send(req.bytes());
        require_send(req.form_data());
    }
}
