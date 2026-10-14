use js_sys::{Array, JsString, Object, Promise, Reflect, Uint8Array};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

#[allow(clippy::all)]
mod env;
use env::Env;

fn text(body: &str, status: f64, content_type: &str) -> Result<Response> {
    let headers = Headers::new()?;
    headers.set("content-type", content_type)?;
    Ok(Response::new_with_str_and_init(
        body,
        &ResponseInit::builder()
            .status(status)
            .headers(&headers)
            .build(),
    )?)
}

#[wasm_bindgen(js_name = "fetch", js_namespace = "default")]
pub async fn fetch_handler(req: Request, env: Env, ctx: ExecutionContext) -> Result<Response> {
    let url = req.url();
    let path = url.splitn(4, '/').nth(3).unwrap_or("");
    let path = path.split('?').next().unwrap_or("");

    match (req.method().as_str(), path) {
        ("GET", "") => text("Hello from Rust!", 200.0, "text/plain"),

        ("GET", "env") => text(&env.greeting(), 200.0, "text/plain"),

        ("GET", "headers") => {
            let headers = Headers::new()?;
            headers.append("x-multi", "a")?;
            headers.append("x-multi", "b")?;
            headers.set("x-single", "1")?;
            headers.set("x-deleted", "gone")?;
            headers.delete("x-deleted")?;
            let body = format!(
                "multi={} single={} has_deleted={} has_single={} incoming={}",
                headers.get("x-multi")?.unwrap_or_default(),
                headers.get("x-single")?.unwrap_or_default(),
                headers.has("x-deleted")?,
                headers.has("x-single")?,
                req.headers().get("x-incoming")?.unwrap_or_default(),
            );
            Ok(Response::new_with_str_and_init(
                &body,
                &ResponseInit::builder().headers(&headers).build(),
            )?)
        }

        ("POST", "echo") => {
            let body: String = req.text().await?.into();
            let name = req.headers().get("x-name")?.unwrap_or_default();
            Ok(Response::new_with_str(&format!("{name}: {body}"))?)
        }

        ("POST", "bytes") => {
            let buf = req.array_buffer().await?;
            let bytes = Uint8Array::new(&buf).to_vec();
            let sum: u32 = bytes.iter().map(|b| *b as u32).sum();
            let out = Uint8Array::from(&bytes[..]);
            let headers = Headers::new()?;
            headers.set("x-sum", &sum.to_string())?;
            Ok(Response::new_with_array_buffer_and_init(
                &out.buffer(),
                &ResponseInit::builder().headers(&headers).build(),
            )?)
        }

        ("POST", "clone") => {
            let copy = req.clone()?;
            let a: String = req.text().await?.into();
            let b: String = copy.text().await?.into();
            Ok(Response::new_with_str(&format!("{a}|{b}"))?)
        }

        ("GET", "json") => {
            let data = Object::new();
            Reflect::set(&data, &"method".into(), &JsString::from(req.method()))?;
            Reflect::set(&data, &"n".into(), &42.0.into())?;
            Ok(Response::json(&data)?)
        }

        ("GET", "redirect") => Ok(Response::redirect_with_status(
            "https://example.com/elsewhere",
            302.0,
        )?),

        ("GET", "fetch") => {
            let upstream = fetch_with_str("https://upstream.test/data").await?;
            let status = upstream.status();
            let ok = upstream.ok();
            let body: String = upstream.text().await?.into();
            text(&format!("{status} {ok} {body}"), 200.0, "text/plain")
        }

        ("POST", "fetch-init") => {
            let body: String = req.text().await?.into();
            let headers = Headers::new()?;
            headers.set("x-forwarded", "yes")?;
            let init = RequestInit::builder()
                .method("PUT")
                .headers(&headers)
                .body_with_str(&body)
                .build();
            let upstream = fetch_with_str_and_init("https://upstream.test/echo", &init).await?;
            let echoed: String = upstream.text().await?.into();
            Ok(Response::new_with_str(&echoed)?)
        }

        ("GET", "wait-until") => {
            ctx.wait_until(&Promise::resolve(&JsValue::NULL))?;
            ctx.pass_through_on_exception()?;
            Ok(Response::new_with_str("ok")?)
        }

        ("GET", "log") => {
            console::log(&[JsValue::from("hello"), 1.0.into()]);
            console::warn(&Array::of1(&"warned".into()).to_vec());
            Ok(Response::new_with_str("logged")?)
        }

        ("GET", "stream") => {
            let body = req.body().is_some();
            Ok(Response::new_with_str(&format!("body={body}"))?)
        }

        ("GET", "bad-header") => {
            let headers = Headers::new()?;
            let outcome = match headers.set("bad name", "x") {
                Err(JsException::TypeError(e)) => {
                    format!("TypeError: {}", String::from(e.message()))
                }
                Err(other) => format!("other: {other}"),
                Ok(()) => "ok".to_string(),
            };
            text(&outcome, 200.0, "text/plain")
        }

        ("GET", "bad-redirect") => {
            let outcome = match Response::redirect_with_status("https://example.com/", 200.0) {
                Err(JsException::RangeError(_)) => "RangeError".to_string(),
                Err(other) => format!("other: {other}"),
                Ok(_) => "ok".to_string(),
            };
            text(&outcome, 200.0, "text/plain")
        }

        ("GET", "error-display") => {
            let err: Error = match Headers::new()?.set("bad name", "x") {
                Err(e) => e.into(),
                Ok(()) => "unexpected".into(),
            };
            let matched = matches!(err, Error::Js(JsException::TypeError(_)));
            text(&format!("{matched} {err}"), 200.0, "text/plain")
        }

        ("GET", "error") => Err(Error::Rust("boom".to_string())),

        _ => text("Not Found", 404.0, "text/plain"),
    }
}
