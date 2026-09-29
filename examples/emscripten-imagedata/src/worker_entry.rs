use serde_json::Value;
use worker::*;

use crate::pipeline::{self, Info, Options, Stage};

const USAGE: &str = "POST an image to sanitise it for model input.

  application/json  multimodal message body: every data:image/... URL and
                    every {\"type\":\"base64\",\"data\":...} source is replaced
                    in place with the sanitised JPEG
  image/*           raw image bytes, returns image/jpeg
  otherwise         a data: URL or bare base64 string, returns a data: URL

Query: max=<longest edge, default 1568>  quality=<1-100, default 85>
       stage=sniff|decode|resize|encode  (stop early, for benchmarking)
       repeat=<n>  run the pipeline n times per image (for benchmarking)
";

struct Params {
    opts: Options,
    repeat: usize,
}

fn params(req: &Request) -> Result<Params> {
    let url = req.url()?;
    let mut opts = Options::default();
    let mut repeat = 1;
    for (k, v) in url.query_pairs() {
        match &*k {
            "repeat" => {
                repeat = v
                    .parse()
                    .map_err(|_| Error::RustError("bad repeat".into()))?
            }
            "max" => opts.max_edge = v.parse().map_err(|_| Error::RustError("bad max".into()))?,
            "quality" => {
                opts.quality = v
                    .parse()
                    .map_err(|_| Error::RustError("bad quality".into()))?
            }
            "stage" => {
                opts.stage = Stage::parse(&v).ok_or_else(|| Error::RustError("bad stage".into()))?
            }
            _ => {}
        }
    }
    Ok(Params {
        opts,
        repeat: repeat.max(1),
    })
}

/// One sanitisation of a base64 payload, repeated for benchmarking. The
/// output is only kept from the final run.
fn sanitise_base64(
    data: &str,
    p: &Params,
) -> std::result::Result<pipeline::Processed, pipeline::Error> {
    for _ in 1..p.repeat {
        let out = pipeline::process(&pipeline::decode_base64(data)?, &p.opts)?;
        std::hint::black_box(pipeline::encode_base64(&out.jpeg));
    }
    pipeline::process(&pipeline::decode_base64(data)?, &p.opts)
}

fn sanitise_bytes(
    input: &[u8],
    p: &Params,
) -> std::result::Result<pipeline::Processed, pipeline::Error> {
    for _ in 1..p.repeat {
        std::hint::black_box(pipeline::process(input, &p.opts)?);
    }
    pipeline::process(input, &p.opts)
}

fn bad_request(e: impl std::fmt::Display) -> Result<Response> {
    Response::error(e.to_string(), 400)
}

fn data_url(jpeg: &[u8]) -> String {
    format!("data:image/jpeg;base64,{}", pipeline::encode_base64(jpeg))
}

/// Replace every embedded image in a multimodal message body in place.
fn walk(
    v: &mut Value,
    p: &Params,
    infos: &mut Vec<Info>,
) -> std::result::Result<(), pipeline::Error> {
    let opts = &p.opts;
    match v {
        Value::String(s) if s.starts_with("data:image/") => {
            let out = sanitise_base64(s, p)?;
            if opts.stage == Stage::Encode {
                *s = data_url(&out.jpeg);
            }
            infos.push(out.info);
        }
        Value::Object(map) => {
            let is_b64_source = map.get("type").and_then(Value::as_str) == Some("base64")
                && map.get("data").is_some_and(Value::is_string);
            if is_b64_source {
                let out = sanitise_base64(map["data"].as_str().unwrap(), p)?;
                if opts.stage == Stage::Encode {
                    map.insert(
                        "data".into(),
                        Value::String(pipeline::encode_base64(&out.jpeg)),
                    );
                    map.insert("media_type".into(), Value::String("image/jpeg".into()));
                }
                infos.push(out.info);
            } else {
                for child in map.values_mut() {
                    walk(child, p, infos)?;
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                walk(child, p, infos)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn with_headers(mut resp: Response, infos: &[Info]) -> Result<Response> {
    let headers = resp.headers_mut();
    headers.set("x-image-info", &serde_json::to_string(infos)?)?;
    let bytes = core::arch::wasm32::memory_size(0) as u64 * 65536;
    headers.set("x-wasm-memory-bytes", &bytes.to_string())?;
    Ok(resp)
}

#[event(fetch)]
async fn fetch(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if req.method() != Method::Post {
        return Response::ok(USAGE);
    }
    let p = params(&req)?;
    let opts = &p.opts;
    let content_type = req
        .headers()
        .get("content-type")?
        .unwrap_or_default()
        .to_ascii_lowercase();

    if content_type.starts_with("application/json") {
        let mut body: Value = match serde_json::from_str(&req.text().await?) {
            Ok(v) => v,
            Err(e) => return bad_request(e),
        };
        let mut infos = Vec::new();
        if let Err(e) = walk(&mut body, &p, &mut infos) {
            return bad_request(e);
        }
        let resp = Response::from_json(&body)?;
        return with_headers(resp, &infos);
    }

    let out = if content_type.starts_with("image/") {
        sanitise_bytes(&req.bytes().await?, &p)
    } else {
        sanitise_base64(&req.text().await?, &p)
    };
    let out = match out {
        Ok(out) => out,
        Err(e) => return bad_request(e),
    };

    let resp = if opts.stage != Stage::Encode {
        Response::from_json(&out.info)?
    } else if content_type.starts_with("image/") {
        let headers = Headers::new();
        headers.set("content-type", "image/jpeg")?;
        Response::from_bytes(out.jpeg)?.with_headers(headers)
    } else {
        Response::ok(data_url(&out.jpeg))?
    };
    with_headers(resp, &[out.info])
}
