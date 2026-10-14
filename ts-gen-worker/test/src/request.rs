use super::ApiData;
use crate::util::{error, from_bytes, from_json, ok, path, sleep, text, url};
use crate::Env;
use crate::SomeSharedData;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen_futures::spawn_local;

fn cf_summary(req: &Request) -> (String, String) {
    let cf = req.cf();
    (
        format!(
            "{:?}",
            cf.as_ref().map(|cf| (
                cf.latitude().unwrap_or_default(),
                cf.longitude().unwrap_or_default()
            ))
        ),
        cf.map_or(String::from("No CF properties"), |cf| {
            cf.region().unwrap_or_else(|| "unknown region".into())
        }),
    )
}

pub fn handle_a_request(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let (located, within) = cf_summary(&req);
    ok(format!(
        "req at: {}, located at: {located}, within: {within}",
        path(&req)?
    ))
}

pub async fn handle_async_request(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let (located, within) = cf_summary(&req);
    ok(format!(
        "[async] req at: {}, located at: {located}, within: {within}",
        path(&req)?
    ))
}

pub async fn handle_test_data(_req: Request, _env: Env, data: SomeSharedData) -> Result<Response> {
    if data.regex.is_match("2014-01-01") {
        ok("data ok")
    } else {
        error("bad match", 500)
    }
}

pub async fn handle_xor(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let url = url(&req)?;
    let num: u8 = match url.path_segments().unwrap().nth(1).unwrap().parse() {
        Ok(num) => num,
        Err(_) => return error("invalid byte", 400),
    };

    let Some(body) = req.body() else {
        return error("missing body", 400);
    };
    let transform = TransformStream::new()?;
    let writer = transform.writable().get_writer()?;
    let reader = body.get_reader()?;
    spawn_local(async move {
        loop {
            let Ok(chunk) = reader.read().await else {
                break;
            };
            if chunk.done() {
                break;
            }
            let bytes = Uint8Array::new(&chunk.value());
            let mut buf = bytes.to_vec();
            buf.iter_mut().for_each(|x| *x ^= num);
            if writer
                .write_with_chunk(&Uint8Array::from(&buf[..]))
                .await
                .is_err()
            {
                break;
            }
        }
        let _ = writer.close().await;
    });

    Ok(Response::new_with_readable_stream(&transform.readable())?)
}

pub async fn handle_headers(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let headers = Headers::new_with_headers(&req.headers())?;
    headers.append("Hello", "World!")?;

    Ok(Response::new_with_str_and_init(
        "returned your headers to you.",
        &ResponseInit::builder().headers(&headers).build(),
    )?)
}

pub async fn handle_post_file_size(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let bytes = req.bytes().await?;
    ok(format!("size = {}", bytes.length()))
}

pub async fn handle_async_text_echo(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    ok(text(&req).await?)
}

pub async fn handle_secret(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    ok(env.some_secret())
}

pub async fn handle_var(_req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    ok(env.some_variable())
}

pub async fn handle_object_var(_req: Request, env: Env, _: SomeSharedData) -> Result<Response> {
    #[derive(Serialize, Deserialize, PartialEq, Eq)]
    struct Obj {
        foo: i32,
        bar: String,
    }
    let obj: Obj = serde_wasm_bindgen::from_value(env.some_object_variable().into())
        .map_err(|e| e.to_string())?;
    from_json(&obj)
}

pub async fn handle_bytes(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    from_bytes(&[1, 2, 3, 4, 5, 6, 7])
}

pub async fn handle_api_data(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let data = req.bytes().await?.to_vec();
    let mut todo: ApiData = match serde_json::from_slice(&data) {
        Ok(todo) => todo,
        Err(e) => {
            return ok(e.to_string());
        }
    };

    unsafe { todo.title.as_mut_vec().reverse() };

    console::log(&[format!("todo = (title {}) (id {})", todo.title, todo.user_id).into()]);

    from_bytes(&serde_json::to_vec(&todo).map_err(|e| e.to_string())?)
}

pub async fn handle_nonsense_repeat(
    _req: Request,
    _env: Env,
    data: SomeSharedData,
) -> Result<Response> {
    if data.regex.is_match("2014-01-01") {
        ok("data ok")
    } else {
        error("bad match", 500)
    }
}

pub async fn handle_status(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = url(&req)?;
    let mut segments = uri.path_segments().unwrap();
    let code = segments.nth(1);
    if let Some(code) = code {
        return match code.parse::<u16>() {
            Ok(status) => Ok(Response::new_with_str_and_init(
                "You set the status code!",
                &ResponseInit::builder().status(f64::from(status)).build(),
            )?),
            Err(_e) => error("Failed to parse your status code.", 400),
        };
    }

    error("Bad Request", 400)
}

pub async fn handle_redirect_default(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Ok(Response::redirect("https://example.com")?)
}

pub async fn handle_redirect_307(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    Ok(Response::redirect_with_status(
        "https://example.com",
        307.0,
    )?)
}

pub async fn handle_now(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let js_date = js_sys::Date::new_0();
    ok(String::from(js_date.to_string()))
}

pub async fn handle_cloned(_req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let resp = Response::new_with_str("Hello")?;
    let resp1 = resp.clone()?;

    let left = resp.text().await?;
    let right = resp1.text().await?;

    ok((left == right).to_string())
}

pub async fn handle_cloned_stream(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let transform = TransformStream::new()?;
    let writer = transform.writable().get_writer()?;
    spawn_local(async move {
        for index in 0..10 {
            sleep(100.0).await;
            let chunk = Uint8Array::from(index.to_string().as_bytes());
            if writer.write_with_chunk(&chunk).await.is_err() {
                return;
            }
        }
        let _ = writer.close().await;
    });

    let resp = Response::new_with_readable_stream(&transform.readable())?;
    let resp1 = resp.clone()?;

    let left = resp.text().await?;
    let right = resp1.text().await?;

    ok((left == right).to_string())
}

pub async fn handle_custom_response_body(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    from_bytes(b"hello")
}

pub async fn handle_wait_delay(req: Request, _env: Env, _data: SomeSharedData) -> Result<Response> {
    let uri = url(&req)?;
    let mut segments = uri.path_segments().unwrap();
    let delay = segments.nth(1);
    let delay: f64 = match delay.unwrap().parse::<u64>() {
        Ok(delay) => delay as f64,
        Err(_) => return error("invalid delay", 400),
    };

    sleep(delay).await;

    ok("Waited!\n")
}
