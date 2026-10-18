//! Dynamic binding lookup. An app's generated `Env` has one typed getter per binding,
//! which is the preferred form; `EnvExt` covers names only known at runtime and
//! `worker`'s `env.secret("X")` / `env.var("X")` style.

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};

use crate::{Error, Result};

/// Implemented for `js_sys::Object`, which every generated `Env` derefs to, so
/// `env.secret("X")` works on the app's `Env` without putting these methods on
/// every JS handle. A bare `JsValue` env is `value.unchecked_ref::<Object>()`.
pub trait EnvExt {
    /// The binding `name` as `T`. Fails when the binding is absent; the cast itself is
    /// unchecked, as binding classes are not reachable by `instanceof` in the runtime.
    fn get<T: JsCast>(&self, name: &str) -> Result<T>;

    /// A secret binding's string value.
    fn secret(&self, name: &str) -> Result<String>;

    /// A plain-text variable's string value.
    fn var(&self, name: &str) -> Result<String>;

    /// A JSON variable deserialized into `T`.
    #[cfg(feature = "serde")]
    fn object_var<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T>;
}

fn lookup(env: &Object, name: &str) -> Result<JsValue> {
    js_sys::Reflect::get(env, name)?
        .ok_or_else(|| Error::Rust(format!("Binding `{name}` is undefined.")))
}

fn string(env: &Object, name: &str) -> Result<String> {
    lookup(env, name)?
        .as_string()
        .ok_or_else(|| Error::Rust(format!("Binding `{name}` is not a string.")))
}

impl EnvExt for Object {
    fn get<T: JsCast>(&self, name: &str) -> Result<T> {
        Ok(lookup(self, name)?.unchecked_into())
    }

    fn secret(&self, name: &str) -> Result<String> {
        string(self, name)
    }

    fn var(&self, name: &str) -> Result<String> {
        string(self, name)
    }

    #[cfg(feature = "serde")]
    fn object_var<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T> {
        Ok(serde_wasm_bindgen::from_value(lookup(self, name)?)?)
    }
}
