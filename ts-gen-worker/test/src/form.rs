use super::SomeSharedData;
use crate::util::{error, from_json, nullable, ok, url};
use crate::Env;
use blake2::Blake2b512;
use blake2::Digest;
use serde::{Deserialize, Serialize};
use ts_gen_worker::*;
use wasm_bindgen::prelude::*;

/// `worker::FormEntry`: `FormData.get()` returns `File | string | null`, which the
/// bindings erase to `JsValue`.
enum FormEntry {
    Field(String),
    File(File),
}

impl FormEntry {
    fn from_js(value: JsValue) -> Option<Self> {
        if let Some(s) = value.as_string() {
            Some(Self::Field(s))
        } else {
            value.dyn_into::<File>().ok().map(Self::File)
        }
    }
}

pub async fn handle_formdata_name(
    req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let form = req.form_data().await?;
    const NAME: &str = "name";
    let bad_request = error("Bad Request", 400);

    if !form.has(NAME)? {
        return bad_request;
    }

    let names: Vec<String> = form
        .get_all(NAME)?
        .into_iter()
        .filter_map(FormEntry::from_js)
        .map(|entry| match entry {
            FormEntry::Field(s) => s,
            FormEntry::File(f) => f.name(),
        })
        .collect();
    if names.len() > 1 {
        return from_json(&serde_json::json!({ "names": names }));
    }

    if let Some(value) = FormEntry::from_js(form.get(NAME)?) {
        match value {
            FormEntry::Field(v) => from_json(&serde_json::json!({ NAME: v })),
            FormEntry::File(_) => bad_request,
        }
    } else {
        bad_request
    }
}

#[derive(Deserialize, Serialize)]
struct FileSize {
    name: String,
    size: u32,
}

pub async fn handle_formdata_file_size(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let form = req.form_data().await?;

    if let Some(entry) = FormEntry::from_js(form.get("file")?) {
        return match entry {
            FormEntry::File(file) => {
                let kv: KVNamespace = env.file_sizes();

                let b = file.bytes().await?.to_vec();
                let record = FileSize {
                    name: file.name(),
                    size: u32::try_from(b.len()).unwrap(),
                };

                let mut hasher = Blake2b512::new();
                hasher.update(&b);
                let hash = hasher.finalize();
                let key = hex::encode(&hash[..]);

                let val = serde_json::to_string(&record).map_err(|e| e.to_string())?;
                kv.put(&key, &val).await?;

                let keys: Vec<serde_json::Value> = kv
                    .list()
                    .await?
                    .keys()
                    .into_iter()
                    .map(|key| serde_json::json!({ "name": key.name(), "expiration": key.expiration() }))
                    .collect();
                from_json(&keys)
            }
            FormEntry::Field(_) => error("Bad Request", 400),
        };
    }

    error("Bad Request", 400)
}

pub async fn handle_formdata_file_size_hash(
    req: Request,
    env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let uri = url(&req)?;
    let mut segments = uri.path_segments().unwrap();
    let hash = segments.nth(1);
    if let Some(hash) = hash {
        let kv = env.file_sizes();
        return match nullable(kv.get(hash).await?).map(String::from) {
            Some(val) => {
                let val: FileSize = serde_json::from_str(&val).map_err(|e| e.to_string())?;
                from_json(&val)
            }
            None => error("Not Found", 404),
        };
    }

    error("Bad Request", 400)
}

pub async fn handle_is_secret(req: Request, env: Env, _data: SomeSharedData) -> Result<Response> {
    let form = req.form_data().await?;
    if let Some(secret) = FormEntry::from_js(form.get("secret")?) {
        match secret {
            FormEntry::Field(name) => {
                // `env.secret(&name)` looks a binding up by runtime name; the generated
                // Env only has per-binding getters.
                let val = js_sys::Reflect::get(&env, &name.into())?;
                return ok(val.as_string().unwrap_or_default());
            }
            FormEntry::File(_) => return error("Bad Request", 400),
        };
    }

    error("Bad Request", 400)
}
